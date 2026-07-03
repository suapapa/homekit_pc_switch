#!/bin/bash
# Open a QR code for the URI that matches sdkconfig.defaults / firmware.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

URI="$(uv run ./generate_qr.py -s)"
echo "$URI"

ENCODED_URI="$(python3 -c 'import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1], safe=""))' "$URI")"
open "https://qr.homin.dev?type=url&url=${ENCODED_URI}"
