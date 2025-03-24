#!/bin/bash

set -e  # Exit on error

APP_NAME="plode"
IDENTIFIER="com.$APP_NAME"
VERSION="0.1"
PKG_NAME="$APP_NAME.pkg"
BUILD_DIR="package"
BIN_DIR="$BUILD_DIR/usr/local/bin"
PLIST_DIR="$BUILD_DIR/Library/LaunchAgents"
PLIST_PATH="$PLIST_DIR/com.$APP_NAME.daemon.plist"
NATIVE_MESSAGING_HOST="com.$APP_NAME.native"
EXT_DIR="$BUILD_DIR/Library/Application Support/Google/Chrome/NativeMessagingHosts"
CHROME_SUPPORT_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"

echo "🔨 Building the Rust app..."
cargo build --release

echo "📁 Creating package structure..."
rm -rf "$BUILD_DIR"
mkdir -p "$BIN_DIR"
mkdir -p "$PLIST_DIR"
mkdir -p "$EXT_DIR"
mkdir -p "$CHROME_SUPPORT_DIR"

echo "📦 Copying binary to $BIN_DIR..."
cp "target/release/$APP_NAME" "$BIN_DIR/"
chmod +x "$BIN_DIR/$APP_NAME"

echo "📝 Creating LaunchAgent plist..."
cat <<EOF > "$PLIST_PATH"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>$IDENTIFIER.daemon</string>
    <key>ProgramArguments</key>
    <array>
      <string>/usr/local/bin/$APP_NAME</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
  </dict>
</plist>
EOF

echo "📦 Setting up Native Messaging host..."
NATIVE_HOST_PATH="/usr/local/bin/$APP_NAME"
NATIVE_MANIFEST="$EXT_DIR/$NATIVE_MESSAGING_HOST.json"

cat <<EOF > "$NATIVE_MANIFEST"
{
  "name": "$NATIVE_MESSAGING_HOST",
  "description": "Native messaging host for $APP_NAME",
  "path": "$NATIVE_HOST_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://knldjmfmopnpolahpmmgbagdohdnhkik/"
  ]
}
EOF
chmod o+r "$NATIVE_MANIFEST"

echo "📂 Copying Native Messaging host JSON to Chrome support directory..."
cp "$NATIVE_MANIFEST" "$CHROME_SUPPORT_DIR/"

echo "📦 Creating macOS .pkg installer..."
pkgbuild --root "$BUILD_DIR" \
    --identifier "$IDENTIFIER" \
    --version "$VERSION" \
    --install-location / \
    --scripts scripts \
    "$PKG_NAME"

echo "✅ Done! Install using: sudo installer -pkg $PKG_NAME -target /"
