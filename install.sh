#!/usr/bin/env bash
set -e

# Install binary using Cargo
cargo install --path .

# Find where Cargo put it
BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
BIN_PATH="$BIN_DIR/refmap"

# Install .desktop entry
DESKTOP_FILE="$HOME/.local/share/applications/refmap.desktop"
mkdir -p "$(dirname "$DESKTOP_FILE")"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=RefMap
Exec=$BIN_PATH
EOF

echo "Installed MyApp desktop entry at $DESKTOP_FILE"