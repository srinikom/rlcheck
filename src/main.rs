use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::time::Duration;
use tokio::time;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    sites: Vec<Site>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Site {
    url: String,
    interval: u64, // in seconds
}

#[derive(Debug, Clone)]
struct SiteState {
    last_hash: Option<String>,
    is_up: bool,
}

async fn check_site(url: &str) -> Result<(bool, String), String> {
    match reqwest::get(url).await {
        Ok(response) => {
            let status = response.status();
            let is_up = status.is_success();
            
            match response.text().await {
                Ok(body) => {
                    let mut hasher = Sha256::new();
                    hasher.update(body.as_bytes());
                    let hash = format!("{:x}", hasher.finalize());
                    Ok((is_up, hash))
                }
                Err(e) => Err(format!("Failed to read response body: {}", e)),
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

async fn monitor_site(site: Site, mut state: SiteState) {
    let interval = Duration::from_secs(site.interval);
    let mut interval_timer = time::interval(interval);
    
    loop {
        interval_timer.tick().await;
        
        println!("\n[{}] Checking...", site.url);
        
        match check_site(&site.url).await {
            Ok((is_up, hash)) => {
                // Check if status changed
                if state.is_up != is_up {
                    if is_up {
                        println!("✅ [{}] Site is UP (was down)", site.url);
                    } else {
                        println!("❌ [{}] Site is DOWN (was up)", site.url);
                    }
                    state.is_up = is_up;
                } else {
                    println!("ℹ️  [{}] Status: {}", site.url, if is_up { "UP" } else { "DOWN" });
                }
                
                // Check if content changed
                if let Some(last_hash) = &state.last_hash {
                    if last_hash != &hash {
                        println!("🔄 [{}] Content CHANGED!", site.url);
                        println!("   Old hash: {}...", &last_hash[..16]);
                        println!("   New hash: {}...", &hash[..16]);
                    } else {
                        println!("   Content unchanged");
                    }
                } else {
                    println!("   First check - baseline established");
                }
                
                state.last_hash = Some(hash);
            }
            Err(e) => {
                println!("❌ [{}] Error: {}", site.url, e);
                if state.is_up {
                    println!("   Site is now DOWN");
                    state.is_up = false;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read config file
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.yaml".to_string());
    
    println!("Reading config from: {}", config_path);
    
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: Config = serde_yaml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    if config.sites.is_empty() {
        eprintln!("No sites configured!");
        return Ok(());
    }
    
    println!("\nMonitoring {} site(s):", config.sites.len());
    for site in &config.sites {
        println!("  - {} (check every {}s)", site.url, site.interval);
    }
    println!("\nStarting monitoring... (Press Ctrl+C to stop)\n");
    
    // Create initial state for each site
    let mut handles = vec![];
    
    for site in config.sites {
        let state = SiteState {
            last_hash: None,
            is_up: true,
        };
        
        let handle = tokio::spawn(monitor_site(site, state));
        handles.push(handle);
    }
    
    // Wait for all monitoring tasks (runs forever)
    for handle in handles {
        let _ = handle.await;
    }
    
    Ok(())
}
