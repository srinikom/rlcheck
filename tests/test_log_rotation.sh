#!/bin/bash
# Test script to demonstrate log rotation

echo "Creating a test log file with 20,005 lines..."
rm -f test.log test.log.*

# Create a log file with just over 20K lines
for i in {1..20005}; do
    echo "Test line $i: website: https://example.com | load_time: 100ms | status: up | size: 1000bytes | content_hash: abc12" >> test.log
done

echo "Created test.log with $(wc -l < test.log) lines"

echo ""
echo "To test log rotation manually:"
echo "1. Run: cargo run -- --log-file test.log"
echo "2. Let it run for a few seconds"
echo "3. Watch for log rotation when it exceeds 20K lines"
echo "4. Check for test.log.1, test.log.2, etc."
echo ""
echo "Expected behavior:"
echo "- test.log will be rotated to test.log.1"
echo "- New test.log will be created"
echo "- Up to 4 files: test.log, test.log.1, test.log.2, test.log.3"
