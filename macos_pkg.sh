#!/bin/bash

set -e  # Exit on error

APP_NAME="plode_web_agent"
IDENTIFIER="com.$APP_NAME"
VERSION="0.2"
PKG_NAME="$APP_NAME.pkg"
BUILD_DIR="package"
BIN_DIR="$BUILD_DIR/usr/local/bin"
PLIST_DIR="$BUILD_DIR/Library/LaunchAgents"
PLIST_PATH="$PLIST_DIR/$IDENTIFIER.daemon.plist"
NATIVE_MESSAGING_HOST="com.$APP_NAME.native"
EXT_DIR="$BUILD_DIR/Library/Application Support/Google/Chrome/NativeMessagingHosts"
CHROME_SUPPORT_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
SCRIPTS_DIR="scripts"

ARCHS=("aarch64-apple-darwin" "x86_64-apple-darwin")
UNIVERSAL_BINARY="target/universal/$APP_NAME"

mkdir -p "target/universal"
mkdir -p "$SCRIPTS_DIR"

# Build for both architectures
echo "🔨 Building Rust app for macOS ARM and Intel..."
for ARCH in "${ARCHS[@]}"; do
  cargo build --release --target "$ARCH"
done

# Create universal binary
echo "🔗 Creating universal binary..."
rustup target add aarch64-apple-darwin x86_64-apple-darwin

echo "🔨 Building the Rust app for both architectures..."
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create -output "$UNIVERSAL_BINARY" \
  "target/aarch64-apple-darwin/release/$APP_NAME" \
  "target/x86_64-apple-darwin/release/$APP_NAME"

# Create package structure
echo "📁 Creating package structure..."
rm -rf "$BUILD_DIR"
mkdir -p "$BIN_DIR"
mkdir -p "$PLIST_DIR"
mkdir -p "$EXT_DIR"
mkdir -p "$CHROME_SUPPORT_DIR"

# Copy binary
cp "$UNIVERSAL_BINARY" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"

# Create LaunchAgent plist with proper daemon settings
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
    <string>--daemon</string>
  </array>
  <key>KeepAlive</key>
  <true/>
  <key>RunAtLoad</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/var/log/$APP_NAME.log</string>
  <key>StandardErrorPath</key>
  <string>/var/log/$APP_NAME.error.log</string>
  </dict>
</plist>
EOF

# Create postinstall script to load the LaunchAgent
cat <<EOF > "$SCRIPTS_DIR/postinstall"
#!/bin/bash

# Get the current user (the one running the installer)
CURRENT_USER=\$(logname 2>/dev/null || echo \$SUDO_USER)
if [ -z "\$CURRENT_USER" ]; then
  CURRENT_USER=\$(stat -f%Su /dev/console)
fi

USER_HOME=\$(dscl . -read /Users/\$CURRENT_USER NFSHomeDirectory | cut -d' ' -f2)
PLIST_PATH="/Library/LaunchAgents/$IDENTIFIER.daemon.plist"
USER_PLIST_PATH="\$USER_HOME/Library/LaunchAgents/$IDENTIFIER.daemon.plist"

# Copy plist to user's LaunchAgents directory
mkdir -p "\$USER_HOME/Library/LaunchAgents"
cp "\$PLIST_PATH" "\$USER_PLIST_PATH"
chown \$CURRENT_USER:staff "\$USER_PLIST_PATH"

# Load the LaunchAgent for the current user
sudo -u \$CURRENT_USER launchctl load "\$USER_PLIST_PATH"

# Enable it to start at login
sudo -u \$CURRENT_USER launchctl enable "gui/\$(id -u \$CURRENT_USER)/$IDENTIFIER.daemon"

echo "✅ $APP_NAME daemon has been installed and started"
exit 0
EOF

chmod +x "$SCRIPTS_DIR/postinstall"

# Set up Native Messaging host
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
cp "$NATIVE_MANIFEST" "$CHROME_SUPPORT_DIR/"

# Create macOS .pkg installer
echo "📦 Creating macOS .pkg installer..."
pkgbuild --root "$BUILD_DIR" \
  --identifier "$IDENTIFIER" \
  --version "$VERSION" \
  --install-location / \
  --scripts scripts \
  "$PKG_NAME"

echo "✅ Done! Install using: sudo installer -pkg $PKG_NAME -target /"
echo "The daemon will start automatically after installation and on every restart."
