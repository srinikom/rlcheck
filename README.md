# rlcheck - Rust live check tool

A Rust program that live checks web endpoints for uptime and content changes.

- Use this to make serverless endpoints always hot/warm.
- Designed to be simple and lightweight to be able to run even on SBCs.

## Features

- Checks if web endpoints are up (HTTP status)
- Detects content changes using MD5 hashing
- Configurable check intervals per site
- YAML configuration file
- Monitors multiple sites concurrently
- Optional logging to file with automatic log rotation
- Log rotation: maintains up to 4 log files with 20K lines each

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

- `url`: The web enpoint URL to monitor (must include protocol: http:// or https://)
- `interval`: How often to check the site (in seconds)

## Usage

```bash
make install
# installs rlcheck binary under: $HOME/.cargo/bin
# make sure $HOME/.cargo/bin is in your $PATH
cp config.yaml config.prod.yaml
# update config.prod.yaml file with url and interval
make prod

# Tested on Mac and Linux
```

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

- **url**: The URL being monitored
- **load_time**: Response time in milliseconds
- **status**: up, down, or error
- **size**: Content size in bytes
- **content_hash**: First 5 characters of the MD5 hash
- **error**: Error message (only for error status)

## Example Output

```
Reading config from: config.yaml

Monitoring 2 site(s):
  - https://example.com (check every 30s)
  - https://httpbin.org/status/200 (check every 60s)

Starting monitoring... (Press Ctrl+C to stop)

url: https://example.com | load_time: 143ms | status: up | size: 18585bytes | content_hash: 05f75

url: https://httpbin.org/status/200 | load_time: 305ms | status: up | size: 0bytes | content_hash: d41d8

url: https://example.com | load_time: 167ms | status: up | size: 18627bytes | content_hash: a3f8d

url: https://httpbin.org/status/200 | load_time: 298ms | status: up | size: 0bytes | content_hash: d41d8

url: https://example.com | load_time: n/a | status: error | error: Request failed: connection timeout
```

## Dependencies

- `reqwest` - HTTP client
- `serde` & `serde_yaml` - Configuration parsing
- `md5` - Content hashing
- `tokio` - Async runtime
- `clap` - Command-line argument parsing

## Binary Optimization

The release binary is optimized for size:
- **Size**: ~1.7MB (stripped)
- **Optimizations**: LTO enabled, size optimization (`opt-level = "z"`), stripped symbols
- **TLS**: Uses rustls instead of native-tls for smaller footprint
- Minimal tokio runtime with only required features

## Notes

- The program runs indefinitely until stopped with Ctrl+C
- Each site is monitored concurrently in its own async task
- Content changes are detected by comparing MD5 hashes of the response body
- All output is on a single line per check for easy parsing and logging
- Content size is reported in bytes for every request
- Errors include the error message directly in the output line

## License

MIT License - see [LICENSE](LICENSE) file for details
