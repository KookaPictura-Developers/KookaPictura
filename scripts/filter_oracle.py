#!/usr/bin/env python3
"""ImageMagick differential oracle for Kooka Pictura tasks M6-E / M7-C / M8-B.

ImageMagick is an independent implementation of several `Filter > Blur /
Sharpen / Noise / Other / Stylize` operators. This script applies one of them
to a raw 8-bit image and emits the result so
`crates/pictura-filters/tests/oracle.rs` can diff it against
`pictura_filters::apply`. It is a *sanity* oracle, not a parity oracle: Adobe's
exact integer math and kernels are closed, and ImageMagick's operators only
approximate some of the Photoshop filters.

Layout: raw 8-bit samples, interleaved `rgb`/`rgba` by default. `--planar`
reads/writes channel planes (`RRR…GGG…BBB…`, matching `PixelBuffer::data`).

Operators are passed through to ImageMagick verbatim via `--im-args`; the named
`--op` (plus flags) is a convenience that builds the same argument list. The
filter -> ImageMagick mapping and the measured/tolerated divergence live in
`crates/pictura-filters/tests/README.md`; the authoritative Magick flags are:

    Filter        ImageMagick
    ------------  -----------------------------------------------
    GaussianBlur  -gaussian-blur 0x{sigma}          (sigma = radius / 3)
    BoxBlur       -statistic mean {N}x{N}           (N = 2*radius + 1)
    MotionBlur    -motion-blur 0x{distance}+{angle}
    Median        -median {radius}
    UnsharpMask   -unsharp {kr}x{sigma}+{amount}+{threshold}
    Maximum       -morphology Dilate Square:{radius}
    Minimum       -morphology Erode Square:{radius}
    Offset        -roll {+h}{+v}                    (wrap == true only)
    Custom        -convolve {kernel} (+ convolve:scale, -evaluate add)
    Emboss        -emboss {radius}x{sigma}          (measured, no equivalent)
    Mosaic        -filter box -resize W/n x H/n ! + -filter point -resize WxH !

`Mosaic` is only an exact block average when the cell divides both image
dimensions; IM's resize window is offset from Pictura's top-left blocks
otherwise. The remaining M8 filters (Crystallize, Facet, Fragment, Mezzotint,
Pointillize, Color Halftone) have no faithful operator and are not diffed.

`Maximum`/`Minimum` note: ImageMagick's `Square:N` parameter is a *radius*, so
the footprint is `(2N+1)²`; pass Pictura's `radius`, not `2*radius+1`.

`UnsharpMask` translates the Pictura parameters to ImageMagick units inside
`unsharp_args` (see the constants there and `tests/README.md`); the amount is a
percentage in Pictura and a fraction in ImageMagick. `Custom` translates the
Pictura divisor/bias inside `convolve_args`: ImageMagick normalizes by the
kernel sum, so the scale is `sum(kernel) / pictura_scale`, and the offset is an
`-evaluate add` of `offset / 255` as a percentage.

Usage:
    python3 scripts/filter_oracle.py version
    python3 scripts/filter_oracle.py apply --size 16x16 --planar \
        --op gaussian --sigma 1.0 IN.rgb OUT.rgb
    python3 scripts/filter_oracle.py apply --size 16x16 --op box --radius 3 \
        IN.rgb OUT.rgb
    python3 scripts/filter_oracle.py apply --size 16x16 --op maximum --radius 2 \
        IN.rgb OUT.rgb
    python3 scripts/filter_oracle.py apply --size 16x16 --op roll \
        --horizontal 3 --vertical 2 IN.rgb OUT.rgb
    python3 scripts/filter_oracle.py apply --size 16x16 --op convolve \
        --kernel "0,0,-1,0,0, 0,-1,4,-1,0, -1,4,20,4,-1, 0,-1,4,-1,0, 0,0,-1,0,0" \
        --kernel-scale 4 --kernel-offset 8 IN.rgb OUT.rgb
    python3 scripts/filter_oracle.py apply --size 16x16 --planar \
        --im-args="-motion-blur 0x5+45" IN.rgb OUT.rgb
    python3 scripts/filter_oracle.py apply --size 16x16 --planar \
        --op mosaic --cell 4 IN.rgb OUT.rgb

`--im-args` must be attached with `=` (`--im-args="-median 1"`) or argparse
mistakes the leading `-` for another option.
"""

from __future__ import annotations

import argparse
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

MAGICK = os.environ.get("MAGICK", "magick")


def im_available() -> bool:
    return shutil.which(MAGICK) is not None


def unsharp_args(args: argparse.Namespace) -> list[str]:
    """Translate Pictura Unsharp Mask parameters to the ImageMagick operator.

    Pictura `amount` is a percentage (100 = 1x the blur difference); ImageMagick
    `amount` is a fraction (1.0 = 1x). Pictura `threshold` is an 8-bit level;
    ImageMagick `threshold` is a fraction of the quantum range. `radius` is
    Pictura's UI radius (3 sigma); the IM kernel radius is left 0 (auto).
    """
    amount = args.amount / 100.0
    threshold = args.threshold / 255.0
    return ["-unsharp", f"0x{args.sigma}+{amount}+{threshold}"]


def convolve_args(args: argparse.Namespace) -> list[str]:
    """Translate a Pictura Custom 5x5 kernel to the ImageMagick operator.

    Pictura computes `Σ kernel·neighbor / scale + offset`. ImageMagick
    `-convolve` first normalizes by the kernel sum, so the matching
    `convolve:scale` is `sum(kernel) / scale`; the additive offset is applied
    afterwards as `-evaluate add` with a percentage of the quantum range
    (`offset / 255`). The kernel is row-major, comma- or semicolon-separated.
    """
    values = [
        float(v)
        for v in args.kernel.replace(";", ",").split(",")
        if v.strip()
    ]
    if len(values) != 25:
        raise SystemExit(f"--kernel must have 25 values, got {len(values)}")
    if args.kernel_scale == 0.0:
        raise SystemExit("--kernel-scale must be non-zero")
    im_scale = sum(values) / args.kernel_scale
    out = [
        "-define",
        f"convolve:scale={im_scale}",
        "-convolve",
        ",".join(repr(v) for v in values),
    ]
    if args.kernel_offset != 0.0:
        out += ["-evaluate", "add", f"{args.kernel_offset / 255.0 * 100.0}%"]
    return out


def build_im_args(args: argparse.Namespace) -> list[str]:
    """Translate a named --op plus flags into Magick operator arguments."""
    op = args.op
    if op is None:
        return shlex.split(args.im_args)

    if op == "gaussian":
        return ["-gaussian-blur", f"0x{args.sigma}"]
    if op == "box":
        n = 2 * args.radius + 1
        return ["-statistic", "mean", f"{n}x{n}"]
    if op == "motion":
        return ["-motion-blur", f"0x{args.distance}+{args.angle}"]
    if op == "median":
        return ["-median", str(args.radius)]
    if op == "unsharp":
        return unsharp_args(args)
    if op == "maximum":
        return ["-morphology", "Dilate", f"Square:{args.radius}"]
    if op == "minimum":
        return ["-morphology", "Erode", f"Square:{args.radius}"]
    if op == "roll":
        return ["-roll", f"{args.horizontal:+d}{args.vertical:+d}"]
    if op == "solarize":
        return ["-solarize", f"{args.threshold_percent}%"]
    if op == "emboss":
        return ["-emboss", f"{args.emboss_radius}x{args.emboss_sigma}"]
    if op == "mosaic":
        w, h = (int(part) for part in args.size.split("x"))
        small_w = max(1, w // args.cell)
        small_h = max(1, h // args.cell)
        return [
            "-filter", "box", "-resize", f"{small_w}x{small_h}!",
            "-filter", "point", "-resize", f"{w}x{h}!",
        ]
    if op == "convolve":
        return convolve_args(args)
    raise SystemExit(f"unknown --op {op!r}")


def interleave(planar: bytes, width: int, height: int, channels: int) -> bytes:
    pixels = width * height
    out = bytearray(len(planar))
    for i in range(pixels):
        for c in range(channels):
            out[i * channels + c] = planar[c * pixels + i]
    return bytes(out)


def planarize(interleaved: bytes, width: int, height: int, channels: int) -> bytes:
    pixels = width * height
    out = bytearray(len(interleaved))
    for i in range(pixels):
        for c in range(channels):
            out[c * pixels + i] = interleaved[i * channels + c]
    return bytes(out)


def cmd_version(_args: argparse.Namespace) -> int:
    result = subprocess.run([MAGICK, "-version"], check=True, capture_output=True, text=True)
    print(result.stdout, end="")
    return 0


def cmd_apply(args: argparse.Namespace) -> int:
    if "x" not in args.size:
        print(f"FAIL: --size must be WxH, got {args.size!r}", file=sys.stderr)
        return 1
    width, height = (int(part) for part in args.size.split("x"))
    channels = args.channels
    expected = width * height * channels

    raw = Path(args.input).read_bytes()
    if len(raw) != expected:
        print(
            f"FAIL: {args.input} has {len(raw)} bytes, expected {expected} "
            f"for {width}x{height}x{channels}",
            file=sys.stderr,
        )
        return 1
    pixels = interleave(raw, width, height, channels) if args.planar else raw

    fmt = "rgba" if channels == 4 else "rgb"
    im_args = build_im_args(args)
    tmp_in = Path(args.input).with_suffix(".oracle-in.raw")
    tmp_out = Path(args.output).with_suffix(".oracle-out.raw")
    tmp_in.write_bytes(pixels)
    cmd = [
        MAGICK,
        "-size",
        args.size,
        "-depth",
        "8",
        f"{fmt}:{tmp_in}",
        *im_args,
        "-depth",
        "8",
        f"{fmt}:{tmp_out}",
    ]
    try:
        subprocess.run(cmd, check=True, capture_output=True, text=True)
        result = tmp_out.read_bytes()
    finally:
        tmp_in.unlink(missing_ok=True)
        tmp_out.unlink(missing_ok=True)

    if len(result) != expected:
        print(
            f"FAIL: ImageMagick wrote {len(result)} bytes, expected {expected}",
            file=sys.stderr,
        )
        return 1
    if args.planar:
        result = planarize(result, width, height, channels)
    Path(args.output).write_bytes(result)
    print(f"wrote {args.output}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="ImageMagick filter oracle for M6-E / M7-C / M8-B")
    sub = parser.add_subparsers(dest="command", required=True)

    p_version = sub.add_parser("version", help="print `magick -version`")
    p_version.set_defaults(func=cmd_version)

    p_apply = sub.add_parser("apply", help="apply an ImageMagick operator to a raw image")
    p_apply.add_argument("--size", default="16x16", help="WxH of the raw image")
    p_apply.add_argument("--channels", type=int, choices=(3, 4), default=3)
    p_apply.add_argument("--planar", action="store_true",
                         help="read/write channel planes instead of interleaved")
    p_apply.add_argument("--op", choices=(
        "gaussian", "box", "motion", "median", "unsharp",
        "maximum", "minimum", "roll", "solarize", "emboss", "mosaic", "convolve",
    ), help="named operator; builds the Magick args below")
    p_apply.add_argument("--im-args", default="",
                         help="verbatim Magick operator arguments (when --op is omitted)")
    p_apply.add_argument("--sigma", type=float, default=1.0,
                         help="Gaussian sigma")
    p_apply.add_argument("--radius", type=int, default=1,
                         help="box radius (N = 2r+1), median radius, or morphology radius")
    p_apply.add_argument("--cell", type=int, default=4,
                         help="Mosaic cell size in pixels (must divide the image)")
    p_apply.add_argument("--distance", type=int, default=5,
                         help="motion blur distance")
    p_apply.add_argument("--angle", type=float, default=0.0,
                         help="motion blur angle in degrees")
    p_apply.add_argument("--amount", type=float, default=100.0,
                         help="unsharp amount as a Pictura percentage")
    p_apply.add_argument("--threshold", type=float, default=0.0,
                         help="unsharp threshold as a Pictura 8-bit level")
    p_apply.add_argument("--horizontal", type=int, default=0,
                         help="roll horizontal shift (positive = right)")
    p_apply.add_argument("--vertical", type=int, default=0,
                         help="roll vertical shift (positive = down)")
    p_apply.add_argument("--threshold-percent", type=float, default=50.0,
                         help="solarize threshold as a percentage of quantum range")
    p_apply.add_argument("--emboss-radius", type=float, default=0.0,
                         help="emboss Gaussian radius (IM)")
    p_apply.add_argument("--emboss-sigma", type=float, default=1.0,
                         help="emboss Gaussian sigma (IM)")
    p_apply.add_argument("--kernel", default="",
                         help="Custom 5x5 kernel, row-major, comma/semicolon separated")
    p_apply.add_argument("--kernel-scale", type=float, default=1.0,
                         help="Custom divisor (Pictura scale)")
    p_apply.add_argument("--kernel-offset", type=float, default=0.0,
                         help="Custom additive bias (Pictura 8-bit offset)")
    p_apply.add_argument("input", help="raw 8-bit input image")
    p_apply.add_argument("output", help="raw 8-bit output image")
    p_apply.set_defaults(func=cmd_apply)

    args = parser.parse_args(argv)
    if not im_available():
        print(f"FAIL: '{MAGICK}' not found on PATH (set MAGICK to override)",
              file=sys.stderr)
        return 1
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
