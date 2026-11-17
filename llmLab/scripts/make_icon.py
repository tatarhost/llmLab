#!/usr/bin/env python3
"""
Generate multi-resolution Windows icon (assets/icon.ico) for llmLab.

1. Reads assets/logo.jpg if present (white geometric robot/chip with </> cutout on black).
2. If assets/logo.jpg is absent, ensures fallback assets/logo.svg exists and renders
   an anti-aliased high-res silhouette using Pillow supersampling.
3. Center-crops into a square with smooth rounded corners.
4. Exports assets/icon.ico containing 16, 32, 48, 64, 128, and 256 px frames.
"""

from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError as exc:
    print(
        "ERROR: Pillow is required. Install it via: pip install pillow",
        file=sys.stderr,
    )
    raise SystemExit(1) from exc


ROOT = Path(__file__).resolve().parent.parent
ASSETS_DIR = ROOT / "assets"
JPG_PATH = ASSETS_DIR / "logo.jpg"
SVG_PATH = ASSETS_DIR / "logo.svg"
ICO_PATH = ASSETS_DIR / "icon.ico"

ICO_SIZES = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

FALLBACK_SVG = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <rect width="512" height="512" rx="96" fill="#05070B"/>
  <path fill="#FFFFFF" d="
    M 124 72
    L 216 72 L 228 98 L 138 98 L 116 146 L 116 190
    L 168 206 L 344 206 L 396 190 L 396 146 L 374 98
    L 284 98 L 296 72 L 388 72 L 434 134 L 434 218
    L 394 246 L 434 282 L 434 440 L 368 440 L 346 376
    L 166 376 L 144 440 L 78 440 L 78 282 L 118 246
    L 78 218 L 78 134 Z
  "/>
  <rect x="132" y="220" width="248" height="140" rx="18" fill="#05070B"/>
  <path fill="#FFFFFF" d="M 212 246 L 156 290 L 212 334 L 228 314 L 194 290 L 228 266 Z"/>
  <path fill="#FFFFFF" d="M 268 238 L 290 238 L 244 342 L 222 342 Z"/>
  <path fill="#FFFFFF" d="M 300 246 L 356 290 L 300 334 L 284 314 L 318 290 L 284 266 Z"/>
</svg>
"""


def render_fallback_image(size: int = 1024) -> Image.Image:
    """Render the fallback geometric robot/chip silhouette with </> cutout at 4x supersampling."""
    if not SVG_PATH.exists():
        SVG_PATH.write_text(FALLBACK_SVG, encoding="utf-8")

    scale = 4
    canvas_size = size * scale
    s = canvas_size / 512.0

    def p(x: float, y: float) -> tuple[float, float]:
        return (x * s, y * s)

    img = Image.new("RGBA", (canvas_size, canvas_size), (5, 7, 11, 255))
    draw = ImageDraw.Draw(img)

    # Outer white robot/chip silhouette
    outer_poly = [
        p(124, 72), p(216, 72), p(228, 98), p(138, 98), p(116, 146), p(116, 190),
        p(168, 206), p(344, 206), p(396, 190), p(396, 146), p(374, 98),
        p(284, 98), p(296, 72), p(388, 72), p(434, 134), p(434, 218),
        p(394, 246), p(434, 282), p(434, 440), p(368, 440), p(346, 376),
        p(166, 376), p(144, 440), p(78, 440), p(78, 282), p(118, 246),
        p(78, 218), p(78, 134),
    ]
    draw.polygon(outer_poly, fill=(255, 255, 255, 255))

    # Inner dark visor cutout
    draw.rounded_rectangle(
        [p(132, 220), p(380, 360)],
        radius=int(18 * s),
        fill=(5, 7, 11, 255),
    )

    # '<' glyph
    draw.polygon(
        [p(212, 246), p(156, 290), p(212, 334), p(228, 314), p(194, 290), p(228, 266)],
        fill=(255, 255, 255, 255),
    )
    # '/' glyph
    draw.polygon(
        [p(268, 238), p(290, 238), p(244, 342), p(222, 342)],
        fill=(255, 255, 255, 255),
    )
    # '>' glyph
    draw.polygon(
        [p(300, 246), p(356, 290), p(300, 334), p(284, 314), p(318, 290), p(284, 266)],
        fill=(255, 255, 255, 255),
    )

    return img.resize((size, size), Image.Resampling.LANCZOS)


def center_crop_square(img: Image.Image, target_size: int = 1024) -> Image.Image:
    """Center-crop an arbitrary image to a square and resize cleanly."""
    img = img.convert("RGBA")
    w, h = img.size
    side = min(w, h)
    left = (w - side) // 2
    top = (h - side) // 2
    cropped = img.crop((left, top, left + side, top + side))
    return cropped.resize((target_size, target_size), Image.Resampling.LANCZOS)


def apply_rounded_corners(img: Image.Image, radius_ratio: float = 0.185) -> Image.Image:
    """Apply anti-aliased rounded corners via 4x supersampled alpha mask."""
    size = img.size[0]
    scale = 4
    mask_big = Image.new("L", (size * scale, size * scale), 0)
    mask_draw = ImageDraw.Draw(mask_big)
    radius = int(size * scale * radius_ratio)
    mask_draw.rounded_rectangle(
        [(0, 0), (size * scale - 1, size * scale - 1)],
        radius=radius,
        fill=255,
    )
    mask = mask_big.resize((size, size), Image.Resampling.LANCZOS)

    out = img.copy()
    existing_alpha = out.getchannel("A")
    combined_alpha = Image.composite(existing_alpha, Image.new("L", (size, size), 0), mask)
    out.putalpha(combined_alpha)
    return out


def main() -> None:
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    if JPG_PATH.exists():
        print(f"Using source logo: {JPG_PATH}")
        with Image.open(JPG_PATH) as src:
            square = center_crop_square(src, target_size=1024)
    else:
        print(f"Source {JPG_PATH} not found; generating fallback icon from SVG silhouette.")
        square = render_fallback_image(size=1024)

    rounded = apply_rounded_corners(square)
    rounded.save(ICO_PATH, format="ICO", sizes=ICO_SIZES)
    print(f"Generated multi-resolution icon: {ICO_PATH} ({len(ICO_SIZES)} sizes)")


if __name__ == "__main__":
    main()
