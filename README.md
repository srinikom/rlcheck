# Website Monitor

A Rust program that monitors websites for uptime and content changes.

## Features

- ✅ Checks if websites are up (HTTP status)
- 🔄 Detects content changes using MD5 hashing
- ⚙️ Configurable check intervals per site
- 📋 YAML configuration file
- 🔄 Monitors multiple sites concurrently
- 📝 Optional logging to file with automatic log rotation
- 🔄 Log rotation: maintains up to 4 log files with 20K lines each

## Configuration

Create a `config.yaml` file with the following format:

```yaml
sites:
  - url: "https://example.com"
    interval: 30  # check every 30 seconds
  - url: "https://another-site.com"
    interval: 60  # check every 60 seconds
```

### Config Fields

- `url`: The website URL to monitor (must include protocol: http:// or https://)
- `interval`: How often to check the site (in seconds)

## Usage

### Build the project

```bash
cargo build --release
```

### Run with default config (config.yaml)

```bash
cargo run
```

### Run with custom config file

```bash
cargo run -- --config path/to/your-config.yaml
```

### Run with logging enabled

```bash
cargo run -- --log-file monitor.log
```

### Command-line options

- `-c, --config <FILE>`: Path to config file (default: config.yaml)
- `-l, --log-file <FILE>`: Path to log file (optional)
- `-h, --help`: Print help information

### Log Rotation

When logging is enabled, the monitor automatically rotates log files:
- Maximum 4 log files maintained (monitor.log, monitor.log.1, monitor.log.2, monitor.log.3)
- Each log file can contain up to 20,000 lines
- When the current log reaches 20K lines, it's rotated:
  - monitor.log → monitor.log.1
  - monitor.log.1 → monitor.log.2
  - monitor.log.2 → monitor.log.3
  - monitor.log.3 is deleted
  - New monitor.log is created

## Output

The program displays simple, structured output on a single line with these key fields:

- **website**: The URL being monitored
- **load_time**: Response time in milliseconds
- **status**: up, down, or error
- **size**: Content size in bytes
- **content_hash**: First 5 characters of the MD5 hash

Additional notifications on separate lines:
- Status changes (up ↔ down)
- Content changes
- Error details

## Example Output

```
Reading config from: config.yaml

Monitoring 2 site(s):
  - https://example.com (check every 30s)
  - https://httpbin.org/status/200 (check every 60s)

Starting monitoring... (Press Ctrl+C to stop)

website: https://example.com | load_time: 143ms | status: up | size: 18585bytes | content_hash: 05f75

website: https://httpbin.org/status/200 | load_time: 305ms | status: up | size: 0bytes | content_hash: d41d8

website: https://example.com | load_time: 167ms | status: up | size: 18627bytes | content_hash: a3f8d
  content changed

website: https://httpbin.org/status/200 | load_time: 298ms | status: up | size: 0bytes | content_hash: d41d8

website: https://example.com | load_time: n/a | status: error
  error: Request failed: connection timeout
  status changed: up -> down
```

## Dependencies

- `reqwest` - HTTP client
- `serde` & `serde_yaml` - Configuration parsing
- `md5` - Content hashing
- `tokio` - Async runtime
- `clap` - Command-line argument parsing

## Notes

- The program runs indefinitely until stopped with Ctrl+C
- Each site is monitored concurrently in its own async task
- Content changes are detected by comparing MD5 hashes of the response body
- Every check displays: URL, load time, status, content size, and MD5 hash on a single line
- Content size is reported in bytes for every request
