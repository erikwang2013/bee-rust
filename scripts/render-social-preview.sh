#!/usr/bin/env bash
# Render docs/social-preview.html to the committed social-preview images.
# Usage: bash scripts/render-social-preview.sh
# Requires google-chrome (headless) and ImageMagick (`magick` or `convert`).
set -eu

cd "$(dirname "$0")/.."

chrome=$(command -v google-chrome || command -v chromium || true)
if [ -z "$chrome" ]; then
    echo "FAIL: google-chrome or chromium not found" >&2
    exit 1
fi
magick=$(command -v magick || command -v convert || true)
if [ -z "$magick" ]; then
    echo "FAIL: ImageMagick (magick or convert) not found" >&2
    exit 1
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
shot="$tmpdir/social-preview.png"

# `?v=A` busts Chrome's cache for the local page; scale factor 1 keeps 1280x630.
"$chrome" --headless=new --disable-gpu --hide-scrollbars \
    --force-device-scale-factor=1 --window-size=1280,630 \
    --screenshot="$shot" "file://$PWD/docs/social-preview.html?v=A"

# Stage next to each destination so the final mv is a rename on the same fs.
for target in docs/social-preview.png site/assets/img/social-preview.png; do
    cp "$shot" "${target%.png}.tmp.png"
    mv "${target%.png}.tmp.png" "$target"
done
for ext in jpg webp; do
    "$magick" "$shot" -quality 90 "site/assets/img/social-preview.tmp.$ext"
    mv "site/assets/img/social-preview.tmp.$ext" "site/assets/img/social-preview.$ext"
done

echo "wrote:"
echo "  docs/social-preview.png"
echo "  site/assets/img/social-preview.png"
echo "  site/assets/img/social-preview.jpg"
echo "  site/assets/img/social-preview.webp"
