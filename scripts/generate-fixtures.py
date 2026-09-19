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
from psd_tools.constants import Tag
from psd_tools.psd.adjustments import (
    BrightnessContrast,
    ColorStop,
    GradientMap,
    LevelRecord,
    Levels,
    PhotoFilter,
    TransparencyStop,
)
from psd_tools.psd.base import EmptyElement, ShortIntegerElement
from psd_tools.psd.descriptor import (
    Descriptor,
    DescriptorBlock,
    Double,
    Enumerated,
    List,
    String,
)
from psd_tools.psd.layer_and_mask import ChannelDataList
from psd_tools.psd.tagged_blocks import TaggedBlock
from psd_tools.terminology import Enum, Key, Type

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


def _adj_layer(psd: PSDImage, key: Tag, name: str, data) -> None:
    """Turn a fresh empty pixel layer into an adjustment layer for `key`."""
    layer = psd.create_pixel_layer(Image.new("RGBA", (2, 2), (0, 0, 0, 0)), name=name)
    rec = layer._record
    layer._channels = ChannelDataList([])
    rec.channel_info = []
    rec.bottom = rec.top
    rec.right = rec.left
    rec.mask_data = None
    rec.tagged_blocks[Tag(key)] = TaggedBlock(key=Tag(key), data=data)


def adjustment() -> PSDImage:
    """RGB, a Base pixel layer plus the adjustment layers the renderer decodes."""
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    _adj_layer(psd, Tag.INVERT, "Invert", EmptyElement())
    _adj_layer(psd, Tag.POSTERIZE, "Posterize", ShortIntegerElement(4))
    _adj_layer(psd, Tag.THRESHOLD, "Threshold", ShortIntegerElement(128))
    _adj_layer(
        psd,
        Tag.BRIGHTNESS_AND_CONTRAST,
        "BrightnessContrast",
        BrightnessContrast(brightness=10, contrast=20, mean=0, lab_only=0),
    )
    recs = [LevelRecord(5, 250, 10, 240, 120)] * 29
    _adj_layer(
        psd, Tag.LEVELS, "Levels", Levels(version=2, extra_version=None, items=recs)
    )
    _adj_layer(
        psd,
        Tag.PHOTO_FILTER,
        "PhotoFilter",
        PhotoFilter(
            version=2,
            color_space=0,
            color_components=(255, 180, 80, 0),
            density=25,
            luminosity=1,
        ),
    )
    return psd


def gradient_map() -> PSDImage:
    """RGB, a Base pixel layer plus a black-to-white Gradient Map adjustment."""
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    _adj_layer(
        psd,
        Tag.GRADIENT_MAP,
        "Gradient Map",
        GradientMap(
            version=1,
            is_reversed=0,
            is_dithered=0,
            name="Black to White",
            color_stops=[
                ColorStop(location=0, midpoint=50, mode=0, color=(0, 0, 0, 0)),
                ColorStop(
                    location=4096,
                    midpoint=50,
                    mode=0,
                    color=(65535, 65535, 65535, 0),
                ),
            ],
            transparency_stops=[
                TransparencyStop(location=0, midpoint=50, opacity=255),
                TransparencyStop(location=4096, midpoint=50, opacity=255),
            ],
            expansion=2,
            interpolation=4096,
            length=32,
            mode=0,
            random_seed=0,
            show_transparency=0,
            use_vector_color=0,
            roughness=0,
            color_model=0,
            minimum_color=[0, 0, 0, 0],
            maximum_color=[0, 0, 0, 0],
        ),
    )
    return psd


def solid_fill() -> PSDImage:
    """RGB, a Base pixel layer plus a SoCo tagged fill layer.

    The tagged-block key is ``SoCo``, but psd-tools' ``DescriptorBlock`` keeps
    its own default class id, so the serialized top-level descriptor class is
    ``null``. pictura's encoder writes ``SoCo`` as the class id, which psd-tools
    accepts identically.
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    data = DescriptorBlock(
        Descriptor(
            {
                b"Clr ": Descriptor(
                    {
                        b"Rd  ": Double(10.0),
                        b"Grn ": Double(20.0),
                        b"Bl  ": Double(30.0),
                    },
                    classID=b"RGBC",
                )
            },
            classID=b"SoCo",
        )
    )
    _adj_layer(psd, Tag.SOLID_COLOR_SHEET_SETTING, "Solid Fill", data)
    return psd


def _gradient_stop(location: int, rgb: tuple[int, int, int]) -> Descriptor:
    """One custom gradient stop for a `GdFl` descriptor's `Clrs` list."""
    return Descriptor(
        {
            Key.Color: Descriptor(
                {
                    Key.Red: Double(float(rgb[0])),
                    Key.Green: Double(float(rgb[1])),
                    Key.Blue: Double(float(rgb[2])),
                },
                classID=b"RGBC",
            ),
            Key.Type: Enumerated(Type.ColorStopType, Enum.UserStop),
            Key.Location: Double(float(location)),
            Key.Midpoint: Double(50.0),
        },
        classID=b"Clrt",
    )


def gradient_fill() -> PSDImage:
    """RGB, a Base pixel layer plus a channel-stripped document-sized GdFl fill.

    The fill layer keeps a document-sized rect so psd-tools composites the ramp
    (a black-to-white 8-pixel row) rather than collapsing it to zero.
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    layer = psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0)), name="Gradient Fill"
    )
    rec = layer._record
    layer._channels = ChannelDataList([])
    rec.channel_info = []
    rec.mask_data = None
    data = DescriptorBlock(
        Descriptor(
            {
                Key.Angle: Double(0.0),
                Key.Type: Enumerated(Type.GradientType, Enum.Linear),
                Key.Gradient: Descriptor(
                    {
                        Key.Name: String("Black to White"),
                        Type.GradientForm: Enumerated(
                            Type.GradientForm, Enum.CustomStops
                        ),
                        b"Intr": Enumerated(Type.Interpolation, b"Lnr "),
                        Key.Colors: List(
                            [
                                _gradient_stop(0, (0, 0, 0)),
                                _gradient_stop(4096, (255, 255, 255)),
                            ]
                        ),
                    },
                    classID=b"Grdn",
                ),
            },
            classID=b"GdFl",
        )
    )
    rec.tagged_blocks[Tag.GRADIENT_FILL_SETTING] = TaggedBlock(
        key=Tag.GRADIENT_FILL_SETTING, data=data
    )
    return psd


FIXTURES = {
    "two_layers.psd": two_layers,
    "group.psd": group,
    "masked.psd": masked,
    "gray.psd": gray,
    "adjustment.psd": adjustment,
    "gradient_map.psd": gradient_map,
    "solid_fill.psd": solid_fill,
    "gradient_fill.psd": gradient_fill,
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
