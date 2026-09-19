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
    Bool,
    Descriptor,
    DescriptorBlock,
    DescriptorBlock2,
    Double,
    Enumerated,
    List,
    String,
    UnitFloat,
)
from psd_tools.psd.layer_and_mask import ChannelDataList
from psd_tools.psd.patterns import (
    Pattern,
    VirtualMemoryArray,
    VirtualMemoryArrayList,
)
from psd_tools.psd.tagged_blocks import TaggedBlock, TaggedBlocks
from psd_tools.terminology import Enum, Key, Klass, Type, Unit

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


def _pattern_channel(data: list[int], depth: int = 8) -> VirtualMemoryArray:
    """One written 2x2 channel for the fixture pattern."""
    channel = VirtualMemoryArray()
    channel.set_data((2, 2), bytes(data), depth, 0)
    return channel


def _fixture_pattern() -> Pattern:
    """A 2x2 RGB pattern: red, green / blue, white, opaque.

    The channel list carries Photoshop's fixed slot layout: three written colour
    slots (``num_channels = 3``), then the two-slot alpha region, the last of
    which holds a written opaque plane. psd-tools' ``get_pattern_color_channels``
    reads the three leading written slots as colour and the trailing one as
    alpha, which is the boundary the Rust decoder follows.
    """
    return Pattern(
        version=1,
        image_mode=3,  # ColorMode.RGB
        point=(2, 2),
        name="Pictura",
        pattern_id="pictura-pattern",
        data=VirtualMemoryArrayList(
            version=3,
            rectangle=(0, 0, 2, 2),
            channels=[
                _pattern_channel([255, 0, 0, 255]),  # R: red, red
                _pattern_channel([0, 255, 0, 255]),  # G: green, white
                _pattern_channel([0, 0, 255, 255]),  # B: blue, white
                VirtualMemoryArray(),  # unwritten alpha slot
                _pattern_channel([255, 255, 255, 255]),  # transparency
            ],
        ),
    )


def _fixture_pattern_16bit() -> Pattern:
    """The same 2x2 RGB pattern with 16-bit channel planes.

    The decoder only reads 8-bit pattern planes, so this pattern is skipped and
    the fill renders the placeholder; the fixture proves a 16-bit payload is not
    misdecoded as 8-bit pixels.
    """
    return Pattern(
        version=1,
        image_mode=3,
        point=(2, 2),
        name="Pictura",
        pattern_id="pictura-pattern-16",
        data=VirtualMemoryArrayList(
            version=3,
            rectangle=(0, 0, 2, 2),
            channels=[
                _pattern_channel([255, 0, 0, 0, 0, 0, 0, 255], 16),
                _pattern_channel([0, 0, 255, 0, 0, 0, 0, 255], 16),
                _pattern_channel([0, 0, 0, 0, 255, 0, 0, 255], 16),
                VirtualMemoryArray(),
                _pattern_channel([255, 0, 255, 0, 255, 0, 255, 0], 16),
            ],
        ),
    )


def _pattern_fill(name: str, pattern: Pattern) -> PSDImage:
    """Base layer plus a document-sized PtFl fill referencing `pattern`.

    The pattern's pixels live in the global `Patt` tagged block
    (`Tag.PATTERNS1`), not in an image resource; the `PtFl` descriptor
    references the pattern by id. The fill layer keeps a document-sized rect so
    psd-tools composites the tile.
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    layer = psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0)), name=name
    )
    rec = layer._record
    layer._channels = ChannelDataList([])
    rec.channel_info = []
    rec.mask_data = None
    data = DescriptorBlock(
        Descriptor(
            {
                b"Ptrn": Descriptor(
                    {
                        b"Nm  ": String("Pictura\x00"),
                        b"Idnt": String(pattern.pattern_id + "\x00"),
                    },
                    classID=b"Ptrn",
                ),
                b"Scl ": UnitFloat(100.0, Unit.Percent),
                b"Algn": Bool(True),
            },
            classID=b"PtFl",
        )
    )
    rec.tagged_blocks[Tag.PATTERN_FILL_SETTING] = TaggedBlock(
        key=Tag.PATTERN_FILL_SETTING, data=data
    )
    psd._record.layer_and_mask_information.tagged_blocks = TaggedBlocks()
    psd.tagged_blocks.set_data(Tag.PATTERNS1, [pattern])
    return psd


def pattern_fill() -> PSDImage:
    """RGB, a Base pixel layer plus a document-sized 8-bit PtFl pattern fill."""
    return _pattern_fill("Pattern Fill", _fixture_pattern())


def pattern_fill_16bit() -> PSDImage:
    """RGB, a Base layer plus a PtFl fill whose pattern planes are 16-bit.

    The decoder skips the 16-bit pattern, so the fill renders the placeholder.
    """
    return _pattern_fill("Pattern Fill 16", _fixture_pattern_16bit())


def drop_shadow() -> PSDImage:
    """RGB, a Base pixel layer plus a `DrSh` drop-shadow pixel layer.

    The effect is the standard object-based `lfx2` `DescriptorBlock2`: a
    top-level `masterFXSwitch` and a `DrSh` object whose keys mirror
    psd-tools' `DropShadow` accessors. `uglg` is off, so the authored local
    `lagl` of 120 is the effective angle.
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    layer = psd.create_pixel_layer(
        Image.new("RGBA", (4, 4), (255, 0, 0, 255)), name="Shadowed", left=0, top=0
    )
    drsh = Descriptor(
        {
            Key.Enabled: Bool(True),
            b"present": Bool(True),
            b"showInDialog": Bool(True),
            Key.Mode: Enumerated(b"BlnM", b"mul "),
            Key.Color: Descriptor(
                {
                    b"Rd  ": Double(10.0),
                    b"Grn ": Double(20.0),
                    b"Bl  ": Double(30.0),
                },
                classID=b"RGBC",
            ),
            Key.Opacity: UnitFloat(75.0, Unit.Percent),
            Key.UseGlobalAngle: Bool(False),
            Key.LocalLightingAngle: UnitFloat(120.0, Unit.Angle),
            Key.Distance: UnitFloat(5.0, Unit.Pixels),
            Key.ChokeMatte: UnitFloat(0.0, Unit.Percent),
            Key.Blur: UnitFloat(5.0, Unit.Pixels),
            Key.TransferSpec: Descriptor(
                {Key.Name: String("Linear")}, classID=b"TrnS"
            ),
            Key.AntiAlias: Bool(True),
            Key.Noise: UnitFloat(0.0, Unit.Percent),
            b"layerConceals": Bool(False),
        },
        classID=b"DrSh",
    )
    layer._record.tagged_blocks[Tag.OBJECT_BASED_EFFECTS_LAYER_INFO] = TaggedBlock(
        key=Tag.OBJECT_BASED_EFFECTS_LAYER_INFO,
        data=DescriptorBlock2(
            {b"masterFXSwitch": Bool(True), b"DrSh": drsh}, classID=Klass.Null
        ),
    )
    return psd


def outer_glow() -> PSDImage:
    """RGB, a Base pixel layer plus an `OrGl` outer-glow pixel layer.

    The effect is the standard object-based `lfx2` `DescriptorBlock2`: a
    top-level `masterFXSwitch` and an `OrGl` object whose keys mirror psd-tools'
    `OuterGlow` accessors. The technique is `GlwT` `BETE`/`PrBL`, the spread is
    `Ckmt` 20, and `blur` (size) is 10.
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    layer = psd.create_pixel_layer(
        Image.new("RGBA", (4, 4), (255, 0, 0, 255)), name="Glowing", left=0, top=0
    )
    orgl = Descriptor(
        {
            Key.Enabled: Bool(True),
            b"present": Bool(True),
            b"showInDialog": Bool(True),
            Key.Mode: Enumerated(b"BlnM", b"scrn"),
            Key.Color: Descriptor(
                {
                    b"Rd  ": Double(40.0),
                    b"Grn ": Double(80.0),
                    b"Bl  ": Double(120.0),
                },
                classID=b"RGBC",
            ),
            Key.Opacity: UnitFloat(60.0, Unit.Percent),
            Key.GlowTechnique: Enumerated(b"BETE", b"PrBL"),
            Key.ChokeMatte: UnitFloat(20.0, Unit.Pixels),
            Key.Blur: UnitFloat(10.0, Unit.Pixels),
            Key.Noise: UnitFloat(0.0, Unit.Percent),
            Key.ShadingNoise: UnitFloat(0.0, Unit.Percent),
            Key.InputRange: UnitFloat(50.0, Unit.Percent),
            Key.AntiAlias: Bool(True),
            Key.TransferSpec: Descriptor(
                {Key.Name: String("Linear")}, classID=b"TrnS"
            ),
        },
        classID=b"OrGl",
    )
    layer._record.tagged_blocks[Tag.OBJECT_BASED_EFFECTS_LAYER_INFO] = TaggedBlock(
        key=Tag.OBJECT_BASED_EFFECTS_LAYER_INFO,
        data=DescriptorBlock2(
            {b"masterFXSwitch": Bool(True), b"OrGl": orgl}, classID=Klass.Null
        ),
    )
    return psd


def inner_shadow() -> PSDImage:
    """RGB, a Base pixel layer plus an `IrSh` inner-shadow pixel layer.

    The effect is the standard object-based `lfx2` `DescriptorBlock2`: a
    top-level `masterFXSwitch` and an `IrSh` object whose keys mirror psd-tools'
    `InnerShadow` accessors. `Ckmt` is the choke (an erode); `uglg` is off, so
    the authored local `lagl` of 120 is the effective angle. There is no
    `layerConceals` key: Inner Shadow has no knock-out control.
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    layer = psd.create_pixel_layer(
        Image.new("RGBA", (4, 4), (255, 0, 0, 255)), name="Inner", left=0, top=0
    )
    irsh = Descriptor(
        {
            Key.Enabled: Bool(True),
            b"present": Bool(True),
            b"showInDialog": Bool(True),
            Key.Mode: Enumerated(b"BlnM", b"mul "),
            Key.Color: Descriptor(
                {
                    b"Rd  ": Double(10.0),
                    b"Grn ": Double(20.0),
                    b"Bl  ": Double(30.0),
                },
                classID=b"RGBC",
            ),
            Key.Opacity: UnitFloat(75.0, Unit.Percent),
            Key.UseGlobalAngle: Bool(False),
            Key.LocalLightingAngle: UnitFloat(120.0, Unit.Angle),
            Key.Distance: UnitFloat(5.0, Unit.Pixels),
            Key.ChokeMatte: UnitFloat(0.0, Unit.Percent),
            Key.Blur: UnitFloat(5.0, Unit.Pixels),
            Key.TransferSpec: Descriptor(
                {Key.Name: String("Linear")}, classID=b"TrnS"
            ),
            Key.AntiAlias: Bool(True),
            Key.Noise: UnitFloat(0.0, Unit.Percent),
        },
        classID=b"IrSh",
    )
    layer._record.tagged_blocks[Tag.OBJECT_BASED_EFFECTS_LAYER_INFO] = TaggedBlock(
        key=Tag.OBJECT_BASED_EFFECTS_LAYER_INFO,
        data=DescriptorBlock2(
            {b"masterFXSwitch": Bool(True), b"IrSh": irsh}, classID=Klass.Null
        ),
    )
    return psd


def inner_glow() -> PSDImage:
    """RGB, a Base pixel layer plus an `IrGl` inner-glow pixel layer.

    The effect is the standard object-based `lfx2` `DescriptorBlock2`: a
    top-level `masterFXSwitch` and an `IrGl` object whose keys mirror psd-tools'
    `InnerGlow` accessors. The source is `glwS` typeID `IGSr` (value `SrcE`); the
    technique is `GlwT` `BETE`/`SfBL`. `Ckmt` is the choke (an erode).
    """
    psd = PSDImage.new("RGB", (WIDTH, HEIGHT), color=(200, 100, 50))
    psd.create_pixel_layer(
        Image.new("RGBA", (WIDTH, HEIGHT), (200, 100, 50, 255)), name="Base"
    )
    layer = psd.create_pixel_layer(
        Image.new("RGBA", (4, 4), (255, 0, 0, 255)), name="Glow", left=0, top=0
    )
    irgl = Descriptor(
        {
            Key.Enabled: Bool(True),
            b"present": Bool(True),
            b"showInDialog": Bool(True),
            Key.Mode: Enumerated(b"BlnM", b"scrn"),
            Key.Color: Descriptor(
                {
                    b"Rd  ": Double(255.0),
                    b"Grn ": Double(255.0),
                    b"Bl  ": Double(255.0),
                },
                classID=b"RGBC",
            ),
            Key.Opacity: UnitFloat(75.0, Unit.Percent),
            Key.GlowTechnique: Enumerated(b"BETE", b"SfBL"),
            Key.ChokeMatte: UnitFloat(0.0, Unit.Percent),
            Key.Blur: UnitFloat(5.0, Unit.Pixels),
            Key.InnerGlowSource: Enumerated(b"IGSr", b"SrcE"),
            Key.Noise: UnitFloat(0.0, Unit.Percent),
            Key.ShadingNoise: UnitFloat(0.0, Unit.Percent),
            Key.InputRange: UnitFloat(50.0, Unit.Percent),
            Key.AntiAlias: Bool(True),
            Key.TransferSpec: Descriptor(
                {Key.Name: String("Linear")}, classID=b"TrnS"
            ),
        },
        classID=b"IrGl",
    )
    layer._record.tagged_blocks[Tag.OBJECT_BASED_EFFECTS_LAYER_INFO] = TaggedBlock(
        key=Tag.OBJECT_BASED_EFFECTS_LAYER_INFO,
        data=DescriptorBlock2(
            {b"masterFXSwitch": Bool(True), b"IrGl": irgl}, classID=Klass.Null
        ),
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
    "pattern_fill.psd": pattern_fill,
    "pattern_fill_16bit.psd": pattern_fill_16bit,
    "drop_shadow.psd": drop_shadow,
    "outer_glow.psd": outer_glow,
    "inner_shadow.psd": inner_shadow,
    "inner_glow.psd": inner_glow,
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
