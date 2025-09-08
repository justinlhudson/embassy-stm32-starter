#!/bin/bash

# Update the chip field in .vscode/launch.json to match the selected board
# Usage: ./update_launch_json <chip_name>

set -e

CHIP_NAME="$1"
LAUNCH_FILE=".vscode/launch.json"

if [[ -z "$CHIP_NAME" ]]; then
    echo "Usage: $0 <chip_name>"
    exit 1
fi

if [[ ! -f "$LAUNCH_FILE" ]]; then
    echo "❌ $LAUNCH_FILE not found."
    exit 1
fi

# Use jq to update the chip field in the first configuration
# If jq is not installed, print an error
if ! command -v jq &> /dev/null; then
    echo "❌ jq is required to update $LAUNCH_FILE. Please install jq (brew install jq)."
    exit 1
fi

# Update the chip field
TMP_FILE="${LAUNCH_FILE}.tmp"
jq --arg chip "$CHIP_NAME" '(.configurations[0].chip) = $chip' "$LAUNCH_FILE" > "$TMP_FILE" && mv "$TMP_FILE" "$LAUNCH_FILE"
echo "✅ Updated $LAUNCH_FILE chip field to $CHIP_NAME"
