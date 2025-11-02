# Website Monitor

A Rust program that monitors websites for uptime and content changes.

## Features

- ✅ Checks if websites are up (HTTP status)
- 🔄 Detects content changes using SHA-256 hashing
- ⚙️ Configurable check intervals per site
- 📋 YAML configuration file
- 🔄 Monitors multiple sites concurrently

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
cargo run -- path/to/your-config.yaml
```

## Output

The program will display:

- ✅ When a site comes UP (was previously down)
- ❌ When a site goes DOWN (was previously up)
- 🔄 When a site's content changes
- ℹ️ Current status on each check
- Content hash comparison (first 16 chars)

## Example Output

```
Reading config from: config.yaml

Monitoring 2 site(s):
  - https://example.com (check every 30s)
  - https://httpbin.org/status/200 (check every 60s)

Starting monitoring... (Press Ctrl+C to stop)

[https://example.com] Checking...
ℹ️  [https://example.com] Status: UP
   First check - baseline established

[https://httpbin.org/status/200] Checking...
ℹ️  [https://httpbin.org/status/200] Status: UP
   First check - baseline established

[https://example.com] Checking...
ℹ️  [https://example.com] Status: UP
🔄 [https://example.com] Content CHANGED!
   Old hash: 3338be695e29...
   New hash: 84983e441c3b...
```

## Dependencies

- `reqwest` - HTTP client
- `serde` & `serde_yaml` - Configuration parsing
- `sha2` - Content hashing
- `tokio` - Async runtime

## Notes

- The program runs indefinitely until stopped with Ctrl+C
- Each site is monitored concurrently in its own async task
- Content changes are detected by comparing SHA-256 hashes of the response body
- The first check for each site establishes a baseline (no change reported)
