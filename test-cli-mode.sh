#!/bin/bash

echo "Testing CLI mode for scheduled backup..."
echo "==========================================="

# Get the development binary path
BINARY_PATH="./src-tauri/target/debug/inlocker"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Please run 'pnpm tauri build --debug' first"
    exit 1
fi

# Test with a sample config ID
CONFIG_ID="test-backup-id"

echo "Running: $BINARY_PATH --backup $CONFIG_ID"
echo "-------------------------------------------"

# Run the binary with CLI args
"$BINARY_PATH" --backup "$CONFIG_ID"

echo "-------------------------------------------"
echo "Test completed. Check the logs above."