use clap::Parser;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Parser, Debug)]
#[command(name = "website-monitor")]
#[command(about = "Monitor websites for uptime and content changes", long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    /// Path to log file (optional)
    #[arg(short, long)]
    log_file: Option<String>,
}

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

struct Logger {
    file: Option<Arc<Mutex<File>>>,
    base_path: Option<PathBuf>,
    current_lines: Arc<Mutex<usize>>,
    max_lines: usize,
    max_files: usize,
}

impl Logger {
    fn new(log_file: Option<String>) -> Self {
        let (file, base_path, line_count) = if let Some(path) = log_file {
            let path_buf = PathBuf::from(&path);
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path_buf)
            {
                Ok(f) => {
                    // Count existing lines
                    let count = count_lines(&path_buf).unwrap_or(0);
                    (Some(Arc::new(Mutex::new(f))), Some(path_buf), count)
                }
                Err(e) => {
                    eprintln!("Failed to open log file {}: {}", path, e);
                    (None, None, 0)
                }
            }
        } else {
            (None, None, 0)
        };

        Logger {
            file,
            base_path,
            current_lines: Arc::new(Mutex::new(line_count)),
            max_lines: 20_000,
            max_files: 4,
        }
    }

    fn log(&self, message: &str) {
        // Always print to console
        println!("{}", message);

        // Write to file if enabled
        if let Some(file) = &self.file {
            let mut current_lines = self.current_lines.lock().unwrap();
            
            // Check if rotation is needed
            if *current_lines >= self.max_lines {
                drop(current_lines); // Release lock before rotation
                self.rotate_logs();
                current_lines = self.current_lines.lock().unwrap();
            }

            if let Ok(mut f) = file.lock() {
                if writeln!(f, "{}", message).is_ok() {
                    *current_lines += 1;
                }
            }
        }
    }

    fn rotate_logs(&self) {
        if let Some(base_path) = &self.base_path {
            let base_str = base_path.to_string_lossy();
            
            // Remove oldest log file if exists (log.3)
            let oldest = format!("{}.{}", base_str, self.max_files - 1);
            let _ = fs::remove_file(&oldest);

            // Rotate existing log files
            for i in (1..self.max_files - 1).rev() {
                let from = format!("{}.{}", base_str, i);
                let to = format!("{}.{}", base_str, i + 1);
                let _ = fs::rename(&from, &to);
            }

            // Move current log to .1
            let first_backup = format!("{}.1", base_str);
            let _ = fs::rename(base_path, &first_backup);

            // Create new log file
            if let Ok(new_file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(base_path)
            {
                if let Some(file) = &self.file {
                    if let Ok(mut f) = file.lock() {
                        *f = new_file;
                    }
                }
                let mut current_lines = self.current_lines.lock().unwrap();
                *current_lines = 0;
            }
        }
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Logger {
            file: self.file.clone(),
            base_path: self.base_path.clone(),
            current_lines: self.current_lines.clone(),
            max_lines: self.max_lines,
            max_files: self.max_files,
        }
    }
}

fn count_lines(path: &Path) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
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

async fn monitor_site(site: Site, mut state: SiteState, logger: Logger) {
    let interval = Duration::from_secs(site.interval);
    let mut interval_timer = time::interval(interval);
    
    loop {
        interval_timer.tick().await;
        
        match check_site(&site.url).await {
            Ok((is_up, hash, content_size, load_time)) => {
                let status = if is_up { "up" } else { "down" };
                let hash_short = &hash[..5.min(hash.len())];
                
                // Simple structured output with hash on same line
                let main_msg = format!("website: {} | load_time: {}ms | status: {} | size: {}bytes | content_hash: {}", 
                         site.url, load_time, status, content_size, hash_short);
                logger.log(&main_msg);
                
                // Check if status changed
                if state.is_up != is_up {
                    if is_up {
                        logger.log("  status changed: down -> up");
                    } else {
                        logger.log("  status changed: up -> down");
                    }
                    state.is_up = is_up;
                }
                
                // Check if content changed
                if let Some(last_hash) = &state.last_hash {
                    if last_hash != &hash {
                        logger.log("  content changed");
                    }
                }
                
                state.last_hash = Some(hash);
                state.last_size = Some(content_size);
            }
            Err(e) => {
                let error_msg = format!("website: {} | load_time: n/a | status: error", site.url);
                logger.log(&error_msg);
                logger.log(&format!("  error: {}", e));
                if state.is_up {
                    state.is_up = false;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();
    
    println!("Reading config from: {}", args.config);
    
    let config_content = fs::read_to_string(&args.config)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: Config = serde_yaml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    if config.sites.is_empty() {
        eprintln!("No sites configured!");
        return Ok(());
    }
    
    // Initialize logger
    let logger = Logger::new(args.log_file.clone());
    
    if let Some(log_path) = &args.log_file {
        println!("Logging to file: {}", log_path);
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
        
        let logger_clone = logger.clone();
        let handle = tokio::spawn(monitor_site(site, state, logger_clone));
        handles.push(handle);
    }
    
    // Wait for all monitoring tasks (runs forever)
    for handle in handles {
        let _ = handle.await;
    }
    
    Ok(())
}
