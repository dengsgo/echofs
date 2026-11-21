#!/usr/bin/env bash
# Generate the committed icon assets from the SVG sources.
#
#   assets/icon.svg (macOS Big Sur grid: 824/1024 squircle + outer margins)
#   -> assets/echofs.icns      every size macOS expects (16..512 @1x/@2x)
#                              compiled with iconutil; used by the .app bundle
#                              (make-macos-app.sh).
#
#   assets/icon-window.svg (full-bleed squircle; Windows shows the bitmap
#   as-is, so the Big Sur margins would render the artwork at ~80% size next
#   to other apps' taskbar icons)
#   -> assets/icon-window.png  a single 256x256 RGBA raster embedded into the
#                              GUI binary (gui.rs) as the eframe runtime window
#                              icon — Windows taskbar / Linux window. macOS
#                              ignores the window icon (the .icns rules there).
#   -> assets/echofs.ico       multi-size (16..256) ICO compiled into the
#                              Windows exe by build.rs (Explorer listing,
#                              pinned shortcuts, Properties dialog).
#
# Run this whenever the icon changes; all outputs are committed so CI does
# not need an SVG rasterizer. Regenerating with a different rasterizer (or
# iconutil for the .icns) rewrites those committed files with equivalent —
# not byte-identical — content.
#
# Requires: rsvg-convert (brew install librsvg) or cairosvg; python3 (stdlib
# only — assembles the .ico). iconutil ships with macOS and is only needed
# for the .icns; everything else is emitted on any platform.
set -euo pipefail

cd "$(dirname "$0")/.."
SRC="assets/icon.svg"             # macOS Big Sur grid
SRC_WIN="assets/icon-window.svg"  # full-bleed window / exe icon
OUT="assets/echofs.icns"
PNG_WIN="assets/icon-window.png"
ICO="assets/echofs.ico"
ICO_SIZES=(16 24 32 48 64 128 256)
ICONSET="$(mktemp -d)/echofs.iconset"
ICO_TMP="$(mktemp -d)"
# Both temp dirs are removed on ANY exit — this trap must be installed before
# the first early `exit` below.
trap 'rm -rf "$ICO_TMP" "$(dirname "$ICONSET")"' EXIT

for f in "$SRC" "$SRC_WIN"; do
  if [[ ! -f "$f" ]]; then
    echo "error: $f not found" >&2
    exit 1
  fi
done

# Pick a rasterizer.
if command -v rsvg-convert >/dev/null 2>&1; then
  render() { rsvg-convert -w "$1" -h "$1" "$2" -o "$3"; }
elif command -v cairosvg >/dev/null 2>&1; then
  render() { cairosvg "$2" -W "$1" -H "$1" -o "$3"; }
else
  echo "error: need rsvg-convert (brew install librsvg) or cairosvg" >&2
  exit 1
fi

# Pick a python for the .ico assembly (stdlib only). Checked before any
# rasterization so a missing interpreter can't leave half-written outputs.
PYTHON="$(command -v python3 || true)"
if [[ -z "$PYTHON" ]]; then
  echo "error: need python3 to assemble $ICO" >&2
  exit 1
fi

mkdir -p "$ICONSET"

# --- window / exe icon (Windows taskbar, Linux window, Windows exe) ---
# Rasterize into $ICO_TMP first so early failures usually leave the repo
# untouched. Updates are not atomic: a later copy or iconutil failure can
# still leave mixed assets; rerun after fixing the error. The 256px render
# doubles as icon-window.png.
for size in "${ICO_SIZES[@]}"; do
  render "$size" "$SRC_WIN" "$ICO_TMP/icon_${size}.png"
done

# Assemble the .ico: BMP/DIB entries for 16..48 (the sizes taskbar/Explorer
# actually request at common DPI; maximum shell compatibility), PNG entries
# for 64..256 (Vista+ standard, what imageres.dll itself ships).
"$PYTHON" - "$ICO" "$ICO_TMP" "${ICO_SIZES[@]}" <<'PYEOF'
import os, struct, sys, zlib

ico_path, tmp = sys.argv[1], sys.argv[2]
sizes = [int(s) for s in sys.argv[3:]]


def load_png(path):
    data = open(path, 'rb').read()
    pos = 8
    idat = b''
    while pos < len(data):
        ln, typ = struct.unpack('>I4s', data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + ln]
        if typ == b'IHDR':
            w, h, bd, ct, _, _, inter = struct.unpack('>IIBBBBB', body)
            assert (bd, ct, inter) == (8, 6, 0), path
        if typ == b'IDAT':
            idat += body
        pos += 12 + ln
    bpp, stride = 4, w * 4
    raw = zlib.decompress(idat)
    px = bytearray(w * h * bpp)
    prev = bytearray(stride)
    i = 0
    for y in range(h):
        ft = raw[i]; i += 1
        line = bytearray(raw[i:i + stride]); i += stride
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            c = prev[x - bpp] if x >= bpp else 0
            if ft == 1:
                line[x] = (line[x] + a) & 255
            elif ft == 2:
                line[x] = (line[x] + b) & 255
            elif ft == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif ft == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else (b if pb <= pc else c))) & 255
        px[y * stride:(y + 1) * stride] = line
        prev = line
    return w, h, bytes(px)


def dib32(w, h, px):
    """32bpp DIB entry: BITMAPINFOHEADER + bottom-up BGRA XOR + 1bpp AND mask."""
    row = w * 4
    xor = bytearray(w * h * 4)
    for y in range(h):
        src = (h - 1 - y) * row  # bottom-up
        dst = y * row
        for x in range(w):
            o = src + x * 4
            xor[dst + x * 4 + 0] = px[o + 2]  # B
            xor[dst + x * 4 + 1] = px[o + 1]  # G
            xor[dst + x * 4 + 2] = px[o + 0]  # R
            xor[dst + x * 4 + 3] = px[o + 3]  # A
    mask_row = ((w + 31) // 32) * 4
    mask = bytearray(mask_row * h)
    for y in range(h):
        src = (h - 1 - y) * row
        for x in range(w):
            if px[src + x * 4 + 3] == 0:  # transparent -> mask bit 1 (MSB first)
                mask[y * mask_row + x // 8] |= 0x80 >> (x % 8)
    hdr = struct.pack('<IiiHHIIiiII', 40, w, h * 2, 1, 32, 0,
                      w * h * 4 + mask_row * h, 0, 0, 0, 0)
    return bytes(hdr) + bytes(xor) + bytes(mask)


images = []
for s in sizes:
    w, h, px = load_png(os.path.join(tmp, f'icon_{s}.png'))
    assert (w, h) == (s, s), (s, w, h)
    if s <= 48:
        images.append((s, dib32(w, h, px)))
    else:
        images.append((s, open(os.path.join(tmp, f'icon_{s}.png'), 'rb').read()))

out = bytearray(struct.pack('<HHH', 0, 1, len(images)))
offset = 6 + 16 * len(images)
for s, blob in images:
    out += struct.pack('<BBBBHHII', s % 256, s % 256, 0, 0, 1, 32, len(blob), offset)
    offset += len(blob)
for s, blob in images:
    out += blob
open(ico_path, 'wb').write(bytes(out))
print(f'wrote {ico_path} ({len(out)} bytes)')
PYEOF

# ICO assembly succeeded; now update the window PNG.
cp "$ICO_TMP/icon_256.png" "$PNG_WIN"
echo "wrote $PNG_WIN"

# --- macOS iconset: each logical size has @1x and @2x variants. Requires
# iconutil (macOS only); skip gracefully elsewhere so the assets above are
# still regenerated. Temp dirs are cleaned by the EXIT trap.
if ! command -v iconutil >/dev/null 2>&1; then
  echo "note: iconutil not found (macOS only) — skipping $OUT" >&2
  exit 0
fi

for size in 16 32 128 256 512; do
  render "$size"           "$SRC" "$ICONSET/icon_${size}x${size}.png"
  render "$((size * 2))"   "$SRC" "$ICONSET/icon_${size}x${size}@2x.png"
done

iconutil -c icns "$ICONSET" -o "$OUT"

echo "wrote $OUT"
