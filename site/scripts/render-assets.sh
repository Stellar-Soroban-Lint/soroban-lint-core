#!/usr/bin/env bash
# Render the binary assets from their SVG sources.
#
# The marks are authored as SVG and committed as SVG, because SVG is the
# reviewable format: a diff shows the geometry. The one asset that cannot be
# SVG is `og.png`, because several link-preview crawlers will not accept an
# image/svg+xml and would render the card as a broken image. So the card is
# authored as `og.svg` and rasterised here.
#
# Rendering is done by headless Chrome rather than by `rsvg-convert` or
# Inkscape: Chrome is what the fonts in these files are specified for, and it
# is what GitHub's own image pipeline uses, so the PNG matches what a browser
# would show. No extra dependency is added to the repository for it.
#
# The card is rendered into a deliberately oversized window and then cropped to
# 1200x630 by scripts/crop-png.mjs. Headless Chrome does not reliably paint an
# SVG all the way to the bottom of a `--window-size` viewport — on Chrome 142 the
# painted area stops near 544 px of a 630 px window, so a line of text at
# y=552 lands half-cut while the background behind it is fine. Rendering with
# headroom and cropping removes the dependency on that number, which would
# otherwise change with every Chrome release.
#
# Usage:
#   scripts/render-assets.sh            # render every asset
#   scripts/render-assets.sh --check    # verify the committed PNG is current
#
# `--check` is what CI runs: a change to og.svg that nobody re-rendered is a
# card that says something the source no longer does.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PUBLIC="$HERE/../public"

CHROME="${CHROME:-}"
if [[ -z "$CHROME" ]]; then
  for candidate in google-chrome chromium chromium-browser; do
    if command -v "$candidate" >/dev/null 2>&1; then
      CHROME="$candidate"
      break
    fi
  done
fi
if [[ -z "$CHROME" ]]; then
  echo "no Chrome/Chromium found. Set CHROME=/path/to/chrome." >&2
  exit 2
fi

CARD_WIDTH=1200
CARD_HEIGHT=630
# Generous headroom so the whole card is inside the painted area on any Chrome.
RENDER_HEIGHT=900

render_og() {
  # Takes the destination as an argument so `--check` can render somewhere
  # temporary. It used to write straight to public/og.png, which meant a failing
  # `--check` overwrote the committed file with a freshly rendered one and left
  # the working tree holding an artefact that was never reviewed.
  local out="$1"
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN

  # Chrome will not screenshot a bare .svg file consistently across versions, so
  # it is wrapped in a minimal HTML document.
  cat > "$tmp/og.html" <<HTML
<!doctype html><meta charset="utf-8">
<style>html,body{margin:0;padding:0;background:#0F0F13}
img{display:block;width:${CARD_WIDTH}px;height:${CARD_HEIGHT}px}</style>
<img src="file://$PUBLIC/og.svg">
HTML

  "$CHROME" --headless --disable-gpu --no-sandbox --hide-scrollbars \
    --force-device-scale-factor=1 \
    --window-size="${CARD_WIDTH},${RENDER_HEIGHT}" \
    --screenshot="$tmp/oversized.png" "$tmp/og.html" >/dev/null 2>&1

  if [[ ! -s "$tmp/oversized.png" ]]; then
    echo "render-assets.sh: Chrome produced no output for og.png" >&2
    exit 1
  fi

  node "$HERE/crop-png.mjs" "$tmp/oversized.png" "$out" "$CARD_WIDTH" "$CARD_HEIGHT" >/dev/null

  if [[ ! -s "$out" ]]; then
    echo "render-assets.sh: cropping produced no output" >&2
    exit 1
  fi
}

if [[ "${1:-}" == "--check" ]]; then
  committed="$PUBLIC/og.png"
  if [[ ! -s "$committed" ]]; then
    echo "og.png is missing. Run scripts/render-assets.sh." >&2
    exit 1
  fi
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  render_og "$tmp/og.png"
  if ! cmp -s "$tmp/og.png" "$committed"; then
    echo "og.png is out of date with og.svg. Run scripts/render-assets.sh and commit the result." >&2
    exit 1
  fi
  echo "assets: og.png matches og.svg"
  exit 0
fi

render_og "$PUBLIC/og.png"
echo "assets: wrote $PUBLIC/og.png (${CARD_WIDTH}x${CARD_HEIGHT})"