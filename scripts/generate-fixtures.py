#!/usr/bin/env python3
"""Author deterministic PSD fixtures with the independent `psd-tools` library.

These fixtures are the oracle for `pictura-codec`: a Rust reader is only
trustworthy if it agrees with a second, independent implementation of the
format. Every pixel value, layer name, and offset below is fixed, and
`psd-tools`/PIL emit byte-identical output on repeated runs, so regenerating
does not churn the working tree.

Regenerate with:  python3 scripts/generate-fixtures.py
"""

from __future__ import annotations

import io
from pathlib import Path

from PIL import Image
from psd_tools import PSDImage

ROOT = Path(__file__).resolve().parent.parent
FIXTURE_DIR = ROOT / "crates" / "pictura-codec" / "tests" / "fixtures"

WIDTH = HEIGHT = 8


def _solid(size: tuple[int, int], color: tuple[int, int, int]) -> Image.Image:
    return Image.new("RGB", size, color)


def two_layers() -> PSDImage:
    """RGB, two named pixel layers with distinct solid colors."""
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(0, 0, 0))
    psd.create_pixel_layer(
        _solid((4, 4), (255, 0, 0)), name="Red", left=0, top=0
    )
    psd.create_pixel_layer(
        _solid((4, 4), (0, 0, 255)), name="Blue", left=4, top=4
    )
    return psd


def group() -> PSDImage:
    """RGB, a group containing two named pixel layers."""
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(0, 0, 0))
    inner_green = psd.create_pixel_layer(
        _solid((WIDTH, HEIGHT), (0, 255, 0)), name="Inner Green"
    )
    inner_yellow = psd.create_pixel_layer(
        _solid((WIDTH, HEIGHT), (255, 255, 0)), name="Inner Yellow"
    )
    psd.create_group([inner_green, inner_yellow], name="Group A")
    return psd


def masked() -> PSDImage:
    """RGB, one pixel layer with a raster layer mask."""
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(0, 0, 0))
    layer = psd.create_pixel_layer(
        _solid((WIDTH, HEIGHT), (128, 64, 32)), name="Masked"
    )
    mask = Image.new("L", (WIDTH, HEIGHT), 0)
    for y in range(HEIGHT):
        for x in range(WIDTH):
            mask.putpixel((x, y), 255 if x >= y else 0)
    layer.create_mask(mask)
    return psd


def gray() -> PSDImage:
    """Grayscale, one named pixel layer."""
    psd = PSDImage.new("L", (WIDTH, HEIGHT), color=0)
    psd.create_pixel_layer(
        Image.new("L", (WIDTH, HEIGHT), 200), name="Gray"
    )
    return psd


FIXTURES = {
    "two_layers.psd": two_layers,
    "group.psd": group,
    "masked.psd": masked,
    "gray.psd": gray,
}


def _dump_tree(layer, indent: int = 0) -> None:
    kind = "group" if layer.is_group() else "pixel"
    mask = " +mask" if layer.has_mask() else ""
    print(f"    {'  ' * indent}{layer.name!r} [{kind}] bbox={layer.bbox}{mask}")
    if layer.is_group():
        for child in layer:
            _dump_tree(child, indent + 1)


def main() -> None:
    FIXTURE_DIR.mkdir(parents=True, exist_ok=True)
    for name, build in FIXTURES.items():
        buffer = io.BytesIO()
        build().save(buffer)
        data = buffer.getvalue()
        path = FIXTURE_DIR / name
        path.write_bytes(data)

        reopened = PSDImage.open(io.BytesIO(data))
        print(f"wrote {path.relative_to(ROOT)} ({len(data)} bytes, "
              f"{reopened.width}x{reopened.height} mode={reopened.color_mode})")
        for layer in reopened:
            _dump_tree(layer)


if __name__ == "__main__":
    main()
