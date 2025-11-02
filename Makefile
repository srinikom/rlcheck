# Makefile for website-monitor

# Variables
CARGO := cargo
BINARY_NAME := website-monitor
CONFIG_FILE := config.yaml

# Default target
.PHONY: all
all: build

# Build the project in release mode
.PHONY: build
build:
	$(CARGO) build --release

# Build the project in debug mode
.PHONY: build-debug
build-debug:
	$(CARGO) build

# Run tests
.PHONY: test
test:
	$(CARGO) test

# Run tests with output
.PHONY: test-verbose
test-verbose:
	$(CARGO) test -- --nocapture

# Run the utility (debug mode)
.PHONY: run
run:
	$(CARGO) run

# Run the utility (release mode)
.PHONY: run-release
run-release:
	$(CARGO) run --release

# Run with a specific config file
.PHONY: run-config
run-config:
	$(CARGO) run -- --config $(CONFIG_FILE)

# Run with logging enabled
.PHONY: run-log
run-log:
	$(CARGO) run -- --log-file monitor.log

# Check the code without building
.PHONY: check
check:
	$(CARGO) check

# Format the code
.PHONY: fmt
fmt:
	$(CARGO) fmt

# Check formatting
.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt -- --check

# Run clippy for linting
.PHONY: clippy
clippy:
	$(CARGO) clippy -- -D warnings

# Clean build artifacts
.PHONY: clean
clean:
	$(CARGO) clean

# Install the binary
.PHONY: install
install:
	$(CARGO) install --path .

# Full CI pipeline: format, clippy, test, build
.PHONY: ci
ci: fmt-check clippy test build

# Watch and rerun on file changes (requires cargo-watch)
.PHONY: watch
watch:
	$(CARGO) watch -x run

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make build         - Build the project in release mode"
	@echo "  make build-debug   - Build the project in debug mode"
	@echo "  make test          - Run tests"
	@echo "  make test-verbose  - Run tests with output"
	@echo "  make run           - Run the utility (debug mode)"
	@echo "  make run-release   - Run the utility (release mode)"
	@echo "  make run-config    - Run with config file"
	@echo "  make run-log       - Run with logging enabled"
	@echo "  make check         - Check code without building"
	@echo "  make fmt           - Format the code"
	@echo "  make fmt-check     - Check code formatting"
	@echo "  make clippy        - Run clippy linter"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make install       - Install the binary"
	@echo "  make ci            - Run full CI pipeline"
	@echo "  make watch         - Watch and rerun on changes"
	@echo "  make help          - Show this help message"
