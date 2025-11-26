#!/bin/bash
# Sutra Desktop - macOS Build Script
# Builds a native macOS application bundle

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
DESKTOP_DIR="$ROOT_DIR/desktop"
BUILD_DIR="$DESKTOP_DIR/build"
APP_NAME="Sutra Desktop"
BUNDLE_ID="ai.sutra.desktop"
VERSION=$(cat "$ROOT_DIR/VERSION" 2>/dev/null || echo "1.0.0")

echo "🏗️  Building Sutra Desktop v${VERSION} for macOS..."

# Clean previous builds
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Build release binary
echo "📦 Building release binary..."
cd "$ROOT_DIR"
cargo build -p sutra-desktop --release

# Create app bundle structure
echo "🎨 Creating app bundle..."
APP_DIR="$BUILD_DIR/${APP_NAME}.app"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary
cp "$ROOT_DIR/target/release/sutra-desktop" "$APP_DIR/Contents/MacOS/Sutra Desktop"

# Create Info.plist
cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>Sutra Desktop</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <false/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
EOF

# Create PkgInfo
echo -n "APPL????" > "$APP_DIR/Contents/PkgInfo"

# Get binary size
BINARY_SIZE=$(du -sh "$APP_DIR/Contents/MacOS/Sutra Desktop" | cut -f1)
APP_SIZE=$(du -sh "$APP_DIR" | cut -f1)

echo ""
echo "✅ Build successful!"
echo ""
echo "📊 Build Summary:"
echo "   Version:     ${VERSION}"
echo "   Binary:      ${BINARY_SIZE}"
echo "   App Bundle:  ${APP_SIZE}"
echo "   Location:    ${APP_DIR}"
echo ""
echo "🚀 To install:"
echo "   cp -r \"${APP_DIR}\" /Applications/"
echo ""
echo "   Or drag '${APP_NAME}.app' to Applications folder."
echo ""

# Optionally create DMG
if command -v hdiutil &> /dev/null; then
    echo "📀 Creating DMG installer..."
    DMG_PATH="$BUILD_DIR/SutraDesktop-${VERSION}-macOS.dmg"
    
    # Create temporary DMG directory
    DMG_TMP="$BUILD_DIR/dmg_tmp"
    mkdir -p "$DMG_TMP"
    cp -r "$APP_DIR" "$DMG_TMP/"
    ln -s /Applications "$DMG_TMP/Applications"
    
    # Create DMG
    hdiutil create -volname "Sutra Desktop" \
        -srcfolder "$DMG_TMP" \
        -ov -format UDZO \
        "$DMG_PATH" 2>/dev/null
    
    rm -rf "$DMG_TMP"
    
    DMG_SIZE=$(du -sh "$DMG_PATH" | cut -f1)
    echo "   DMG:         ${DMG_SIZE}"
    echo "   Location:    ${DMG_PATH}"
    echo ""
fi

echo "✨ Done!"
