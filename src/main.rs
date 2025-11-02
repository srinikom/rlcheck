use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{Duration, Instant};
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
    last_size: Option<usize>,
    is_up: bool,
}

async fn check_site(url: &str) -> Result<(bool, String, usize, u128), String> {
    let start = Instant::now();
    
    match reqwest::get(url).await {
        Ok(response) => {
            let status = response.status();
            let is_up = status.is_success();
            
            match response.text().await {
                Ok(body) => {
                    let load_time = start.elapsed().as_millis();
                    let content_size = body.len();
                    let hash = format!("{:x}", md5::compute(body.as_bytes()));
                    Ok((is_up, hash, content_size, load_time))
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
        
        match check_site(&site.url).await {
            Ok((is_up, hash, content_size, load_time)) => {
                let status = if is_up { "up" } else { "down" };
                let hash_short = &hash[..5.min(hash.len())];
                
                // Simple structured output with hash on same line
                println!("website: {} | load_time: {}ms | status: {} | size: {}bytes | content_hash: {}", 
                         site.url, load_time, status, content_size, hash_short);
                
                // Check if status changed
                if state.is_up != is_up {
                    if is_up {
                        println!("  status changed: down -> up");
                    } else {
                        println!("  status changed: up -> down");
                    }
                    state.is_up = is_up;
                }
                
                // Check if content changed
                if let Some(last_hash) = &state.last_hash {
                    if last_hash != &hash {
                        println!("  content changed");
                    }
                }
                
                state.last_hash = Some(hash);
                state.last_size = Some(content_size);
            }
            Err(e) => {
                println!("website: {} | load_time: n/a | status: error", site.url);
                println!("  error: {}", e);
                if state.is_up {
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
            last_size: None,
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
