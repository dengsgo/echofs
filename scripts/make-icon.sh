#!/usr/bin/env bash
# Generate assets/echofs.icns from assets/icon.svg.
#
# Produces every size macOS expects (16..512 @1x/@2x) into a temporary
# .iconset, then compiles it with iconutil. Run this whenever the icon
# changes; the resulting echofs.icns is committed so CI does not need an
# SVG rasterizer.
#
# Requires: rsvg-convert (brew install librsvg) and iconutil (ships with macOS).
set -euo pipefail

cd "$(dirname "$0")/.."
SRC="assets/icon.svg"
OUT="assets/echofs.icns"
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

# macOS iconset: each logical size has @1x and @2x variants.
for size in 16 32 128 256 512; do
  render "$size"           "$ICONSET/icon_${size}x${size}.png"
  render "$((size * 2))"   "$ICONSET/icon_${size}x${size}@2x.png"
done

iconutil -c icns "$ICONSET" -o "$OUT"
rm -rf "$(dirname "$ICONSET")"

echo "wrote $OUT"
