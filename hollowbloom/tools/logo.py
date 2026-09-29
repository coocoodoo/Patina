#!/usr/bin/env python3
"""Makes the title screen's logo: art/logo-source.jpg -> art/logo-HEIGHT.png.

    tools/logo.py        # from the hollowbloom folder

The source is the logo on a plain green. The green is keyed out by flooding in from the edges
over everything close to the corners' colour, so it stops at the logo's cream outline and the
green leaves inside stay. What's left is scaled down to each height the title screen uses
(averaging each block), and every pixel is set to the nearest of the game's Resurrect 32
colours (by CIELAB distance): the game shows nothing else. Needs numpy, scipy and Pillow.
"""

import os

import numpy as np
from PIL import Image
from scipy import ndimage

SOURCE = os.path.join("art", "logo-source.jpg")
# The heights the title picks between, largest first that fits (see draw_title).
HEIGHTS = (150, 116, 96)
# How close to the backdrop's colour a pixel must be to be flooded away (RGB distance).
TOLERANCE = 70
# Pixels shaved off the logo's edge, where the JPEG smudged it into the green.
FRINGE = 4

PALETTE = [
    0xFFFFFF, 0xFB6B1D, 0xE83B3B, 0x831C5D, 0xC32454, 0xF04F78, 0xF68181, 0xFCA790,
    0xE3C896, 0xAB947A, 0x966C6C, 0x625565, 0x3E3546, 0x0B5E65, 0x0B8A8F, 0x1EBC73,
    0x91DB69, 0xFBFF86, 0xFBB954, 0xCD683D, 0x9E4539, 0x7A3045, 0x6B3E75, 0x905EA9,
    0xA884F3, 0xEAADED, 0x8FD3FF, 0x4D9BE6, 0x4D65B4, 0x484A77, 0x30E1B9, 0x8FF8E2,
]


def lab(rgb):
    """sRGB (0-255) to CIELAB."""
    c = rgb / 255.0
    c = np.where(c > 0.04045, ((c + 0.055) / 1.055) ** 2.4, c / 12.92)
    m = np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]])
    xyz = c @ m.T / np.array([0.95047, 1.0, 1.08883])
    f = np.where(xyz > 0.008856, np.cbrt(xyz), 7.787 * xyz + 16 / 116)
    return np.stack(
        [116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])], -1
    )


def main():
    pal = np.array([[(c >> 16) & 255, (c >> 8) & 255, c & 255] for c in PALETTE], dtype=float)
    pal_lab = lab(pal)
    src = np.asarray(Image.open(SOURCE).convert("RGB")).astype(float)

    # Key out the backdrop: everything near its colour that the edges can reach.
    corners = np.concatenate(
        [src[:30, :30], src[:30, -30:], src[-30:, :30], src[-30:, -30:]]
    ).reshape(-1, 3)
    key = corners.mean(0)
    near = np.sqrt(((src - key) ** 2).sum(-1)) < TOLERANCE
    parts, _ = ndimage.label(near)
    edge = set(np.unique(np.concatenate([parts[0], parts[-1], parts[:, 0], parts[:, -1]])))
    edge.discard(0)
    backdrop = np.isin(parts, list(edge))
    # Pockets of backdrop showing through the logo (the gaps in the H): shut in, but ringed
    # by the same cream outline as the backdrop, where a leaf's green has a dark outline.
    ring = ndimage.binary_dilation(backdrop, iterations=4) & ~ndimage.binary_dilation(
        backdrop, iterations=1
    )
    outline = np.median(src[ring], axis=0)
    for part, box in enumerate(ndimage.find_objects(parts), start=1):
        if part in edge or box is None:
            continue
        ys = slice(max(box[0].start - 4, 0), box[0].stop + 4)
        xs = slice(max(box[1].start - 4, 0), box[1].stop + 4)
        m = parts[ys, xs] == part
        r = ndimage.binary_dilation(m, iterations=3) & ~ndimage.binary_dilation(m, iterations=1)
        if r.any() and (np.sqrt(((src[ys, xs][r] - outline) ** 2).sum(-1)) < 60).mean() >= 0.5:
            backdrop[ys, xs] |= m
    logo = ndimage.binary_erosion(~backdrop, iterations=FRINGE)
    ys, xs = np.nonzero(logo)
    y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    logo, src = logo[y0:y1, x0:x1], src[y0:y1, x0:x1]
    h0, w0 = logo.shape

    for th in HEIGHTS:
        tw = round(w0 * th / h0)
        rows = np.linspace(0, h0, th + 1).astype(int)
        cols = np.linspace(0, w0, tw + 1).astype(int)
        out = np.zeros((th, tw, 4), dtype=np.uint8)
        for i in range(th):
            for j in range(tw):
                m = logo[rows[i] : rows[i + 1], cols[j] : cols[j + 1]]
                # Half covered or more: a pixel of the logo, in the nearest palette colour.
                if m.mean() >= 0.5:
                    avg = src[rows[i] : rows[i + 1], cols[j] : cols[j + 1]][m].mean(0)
                    k = ((lab(avg) - pal_lab) ** 2).sum(-1).argmin()
                    out[i, j, :3] = pal[k].astype(np.uint8)
                    out[i, j, 3] = 255
        path = os.path.join("art", f"logo-{th}.png")
        Image.fromarray(out, "RGBA").save(path, optimize=True)
        print(f"{path}: {tw}x{th}")


if __name__ == "__main__":
    main()
