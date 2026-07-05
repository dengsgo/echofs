#!/usr/bin/env bash
# Build a macOS .app bundle (and optional .dmg) for the EchoFS desktop GUI.
#
#   scripts/make-macos-app.sh [--dmg] [--target <triple>]
#
# Without --target, builds a universal (x86_64 + arm64) binary via `lipo` →
# EchoFS-universal.dmg. With --target, builds that single arch →
# EchoFS-darwin-amd64.dmg / EchoFS-darwin-arm64.dmg. CI builds the two arches
# separately (so Intel and Apple Silicon ship as distinct downloads). Either
# way the bundle is EchoFS.app under target/macos/; pass --dmg to also produce
# a drag-to-Applications disk image.
#
# The bundle is ad-hoc codesigned (`codesign -s -`) so it launches on Apple
# Silicon. It is NOT notarized — see README "Desktop GUI" for the Gatekeeper
# first-launch note. Requires: cargo, lipo, codesign (Xcode CLT), hdiutil.
set -euo pipefail

cd "$(dirname "$0")/.."

APP_NAME="EchoFS"
BIN_NAME="echofs"
BUNDLE_ID="com.dengsgo.echofs"
ICNS="assets/echofs.icns"
OUT_DIR="target/macos"
APP_DIR="$OUT_DIR/$APP_NAME.app"

VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')"

# Map a Rust target triple to the download label used in artifact filenames.
arch_label() {
  case "$1" in
    x86_64-apple-darwin)  echo "darwin-amd64" ;;
    aarch64-apple-darwin) echo "darwin-arm64" ;;
    *)                    echo "$1" ;;
  esac
}

MAKE_DMG=0
SINGLE_TARGET=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dmg) MAKE_DMG=1; shift ;;
    --target) SINGLE_TARGET="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

# --- 1. Build the binary/binaries -----------------------------------------
echo ">> Building $BIN_NAME (gui) v$VERSION"
if [[ -n "$SINGLE_TARGET" ]]; then
  rustup target add "$SINGLE_TARGET" >/dev/null 2>&1 || true
  cargo build --release --features gui --target "$SINGLE_TARGET"
  BIN_PATH="target/$SINGLE_TARGET/release/$BIN_NAME"
  LABEL="$(arch_label "$SINGLE_TARGET")"
else
  # Universal: build both arches and lipo them together.
  rustup target add x86_64-apple-darwin aarch64-apple-darwin >/dev/null 2>&1 || true
  cargo build --release --features gui --target x86_64-apple-darwin
  cargo build --release --features gui --target aarch64-apple-darwin
  mkdir -p "$OUT_DIR"
  BIN_PATH="$OUT_DIR/$BIN_NAME-universal"
  lipo -create \
    "target/x86_64-apple-darwin/release/$BIN_NAME" \
    "target/aarch64-apple-darwin/release/$BIN_NAME" \
    -output "$BIN_PATH"
  LABEL="universal"
  echo ">> Universal binary: $(lipo -archs "$BIN_PATH")"
fi

# --- 2. Assemble the .app layout ------------------------------------------
echo ">> Assembling $APP_DIR"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "$BIN_PATH" "$APP_DIR/Contents/MacOS/$BIN_NAME"
chmod +x "$APP_DIR/Contents/MacOS/$BIN_NAME"

if [[ -f "$ICNS" ]]; then
  cp "$ICNS" "$APP_DIR/Contents/Resources/echofs.icns"
else
  echo "warning: $ICNS missing — run scripts/make-icon.sh; bundle will have no icon" >&2
fi

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>            <string>$APP_NAME</string>
  <key>CFBundleDisplayName</key>     <string>$APP_NAME</string>
  <key>CFBundleIdentifier</key>      <string>$BUNDLE_ID</string>
  <key>CFBundleVersion</key>         <string>$VERSION</string>
  <key>CFBundleShortVersionString</key> <string>$VERSION</string>
  <key>CFBundlePackageType</key>     <string>APPL</string>
  <key>CFBundleExecutable</key>      <string>$BIN_NAME</string>
  <key>CFBundleIconFile</key>        <string>echofs</string>
  <key>LSMinimumSystemVersion</key>  <string>10.15</string>
  <key>NSHighResolutionCapable</key> <true/>
  <key>NSHumanReadableCopyright</key> <string>Apache-2.0</string>
</dict>
</plist>
PLIST

plutil -lint "$APP_DIR/Contents/Info.plist" >/dev/null

# --- 3. Ad-hoc codesign ----------------------------------------------------
# Required so the bundle launches on Apple Silicon. Not a Developer ID
# signature, so Gatekeeper still warns on first launch (documented in README).
echo ">> Ad-hoc codesigning"
codesign --force --deep --sign - --timestamp=none "$APP_DIR"
codesign --verify --deep --strict "$APP_DIR" && echo ">> Signature verified"

echo ">> Built $APP_DIR"

# --- 4. Standalone GUI binary tarball -------------------------------------
# Ship the bare `echofs` (gui) binary too, for users who want the executable
# without the .app wrapper. Named echofs-<label>-gui.tar.gz to match the
# Linux/Windows GUI artifacts. Stage via a copy so the archive contains just
# `echofs` with no path prefix (portable across BSD/GNU tar).
TARBALL="$OUT_DIR/$BIN_NAME-$LABEL-gui.tar.gz"
echo ">> Packaging $TARBALL"
STAGE_BIN="$(mktemp -d)"
cp "$BIN_PATH" "$STAGE_BIN/$BIN_NAME"
chmod +x "$STAGE_BIN/$BIN_NAME"
tar czf "$TARBALL" -C "$STAGE_BIN" "$BIN_NAME"
rm -rf "$STAGE_BIN"
echo ">> Built $TARBALL"

# --- 5. Optional .dmg ------------------------------------------------------
if [[ "$MAKE_DMG" == "1" ]]; then
  DMG_PATH="$OUT_DIR/$APP_NAME-$LABEL.dmg"
  STAGING="$(mktemp -d)/dmg"
  echo ">> Building $DMG_PATH"
  mkdir -p "$STAGING"
  cp -R "$APP_DIR" "$STAGING/"
  ln -s /Applications "$STAGING/Applications"
  rm -f "$DMG_PATH"
  hdiutil create -volname "$APP_NAME" -srcfolder "$STAGING" \
    -ov -format UDZO "$DMG_PATH" >/dev/null
  rm -rf "$(dirname "$STAGING")"
  echo ">> Built $DMG_PATH"
fi
