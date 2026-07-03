#!/usr/bin/env bash
# Generate assets/echofs.icns and assets/icon.png from assets/icon.svg.
#
# - echofs.icns: every size macOS expects (16..512 @1x/@2x) compiled with
#   iconutil; used by the .app bundle (make-macos-app.sh).
# - icon.png: a single 256x256 RGBA raster embedded into the GUI binary
#   (gui.rs) as the egui runtime window icon — this is what gives the
#   Windows taskbar and Linux window their icon.
#
# Run this whenever the icon changes; both outputs are committed so CI does
# not need an SVG rasterizer.
#
# Requires: rsvg-convert (brew install librsvg) and iconutil (ships with macOS).
# iconutil is only needed for the .icns; the .png is emitted on any platform.
set -euo pipefail

cd "$(dirname "$0")/.."
SRC="assets/icon.svg"
OUT="assets/echofs.icns"
PNG="assets/icon.png"
PNG_SIZE=256
ICONSET="$(mktemp -d)/echofs.iconset"

if [[ ! -f "$SRC" ]]; then
  echo "error: $SRC not found" >&2
  exit 1
fi

# Pick a rasterizer.
if command -v rsvg-convert >/dev/null 2>&1; then
  render() { rsvg-convert -w "$1" -h "$1" "$SRC" -o "$2"; }
elif command -v cairosvg >/dev/null 2>&1; then
  render() { cairosvg "$SRC" -W "$1" -H "$1" -o "$2"; }
else
  echo "error: need rsvg-convert (brew install librsvg) or cairosvg" >&2
  exit 1
fi

mkdir -p "$ICONSET"

# egui runtime window icon (Windows taskbar / Linux window). Single square RGBA.
render "$PNG_SIZE" "$PNG"
echo "wrote $PNG"

# macOS iconset: each logical size has @1x and @2x variants. Requires iconutil
# (macOS only); skip gracefully elsewhere so icon.png can still be regenerated.
if ! command -v iconutil >/dev/null 2>&1; then
  echo "note: iconutil not found (macOS only) — skipping $OUT" >&2
  rm -rf "$(dirname "$ICONSET")"
  exit 0
fi

for size in 16 32 128 256 512; do
  render "$size"           "$ICONSET/icon_${size}x${size}.png"
  render "$((size * 2))"   "$ICONSET/icon_${size}x${size}@2x.png"
done

iconutil -c icns "$ICONSET" -o "$OUT"
rm -rf "$(dirname "$ICONSET")"

echo "wrote $OUT"
