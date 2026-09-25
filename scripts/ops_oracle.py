#!/usr/bin/env python3
"""ImageMagick differential oracle for Kooka Pictura task M10-B (image ops).

Pictura's `Image > Image Size / Canvas Size / Image Rotation` operations
(`pictura_ops`) are approximations of closed Photoshop paths. This script
applies one ImageMagick operation to a raw 8-bit image and emits the result so
`crates/pictura-ops/tests/oracle.rs` can diff it against the Rust implementation.
It is a *sanity* oracle, not a parity oracle.

Layout: raw 8-bit samples, interleaved `rgb`/`rgba` by default. `--planar`
reads/writes channel planes (`RRR…GGG…BBB…`, matching `PixelBuffer::data`).

The `resize`/`rotate_arbitrary` ops change the output size, so unlike the
filter oracle this script asks ImageMagick for the resulting dimensions and
prints them as `WxH` on stdout; the output raw file holds the IM-sized result.

    Op (Pictura)       ImageMagick
    ----------------   ------------------------------------------------
    resize Nearest     -filter point    -resize WxH!
    resize Bilinear    -filter triangle -resize WxH!
    resize Bicubic     -filter catrom   -resize WxH!   (Keys a=-0.5)
    resize_canvas      -background 'rgba(r,g,b,a)' -gravity G -extent WxH
    rotate90_cw        -rotate 90
    rotate90_ccw       -rotate 270
    rotate180          -rotate 180
    flip_horizontal    -flop
    flip_vertical      -flip
    rotate_arbitrary   -filter triangle -background 'rgba(...)' -rotate ANGLE

ImageMagick's `-filter cubic` is a B-spline cubic, not Catmull-Rom; the measured
match for Pictura's `Resample::Bicubic` is `-filter catrom`, so the named
`resize` op uses `--filter catrom` by default. `rotate_arbitrary` is compared on
the central region only: IM pads the rotated bounding box by one row/column per
side, and its edge sampler diverges from Pictura's bilinear map (the bbox and
sampling divergences live in `crates/pictura-ops/tests/README.md`).

Usage:
    python3 scripts/ops_oracle.py version
    python3 scripts/ops_oracle.py apply --size 16x16 --planar \
        --op resize --width 32 --height 32 --filter catrom IN.rgb OUT.rgb
    python3 scripts/ops_oracle.py apply --size 16x16 --planar \
        --op resize_canvas --width 20 --height 18 --gravity center \
        --background 10,20,30,255 IN.rgb OUT.rgb
    python3 scripts/ops_oracle.py apply --size 16x12 --planar \
        --op rotate90_cw IN.rgb OUT.rgb
    python3 scripts/ops_oracle.py apply --size 16x16 --planar \
        --op rotate_arbitrary --angle 30 --filter triangle \
        --background 0,0,0,0 IN.rgb OUT.rgb
    python3 scripts/ops_oracle.py apply --size 16x16 --planar \
        --im-args="-rotate 90" IN.rgb OUT.rgb

`--background` takes `r,g,b,a` with an 8-bit alpha; it is formatted as an
ImageMagick `rgba()` string. `--im-args` must be attached with `=`
(`--im-args="-rotate 90"`) or argparse mistakes the leading `-` for an option.
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

GRAVITIES = (
    "northwest",
    "north",
    "northeast",
    "west",
    "center",
    "east",
    "southwest",
    "south",
    "southeast",
)


def im_available() -> bool:
    return shutil.which(MAGICK) is not None


def background_arg(spec: str) -> str:
    """Parse `r,g,b,a` (a is 0..255) into an ImageMagick `rgba()` string."""
    parts = [p.strip() for p in spec.replace(";", ",").split(",") if p.strip()]
    if len(parts) != 4:
        raise SystemExit(f"--background must be r,g,b,a (4 values), got {spec!r}")
    r, g, b, a = (int(p) for p in parts)
    return f"rgba({r},{g},{b},{a / 255.0:.6f})"


def build_im_args(args: argparse.Namespace) -> list[str]:
    """Translate a named --op plus flags into Magick operator arguments."""
    op = args.op
    if op is None:
        return shlex.split(args.im_args)

    if op == "resize":
        if args.width is None or args.height is None:
            raise SystemExit("--op resize requires --width and --height")
        return shlex.split(args.im_args) + [
            "-filter", args.filter, "-resize", f"{args.width}x{args.height}!",
        ]
    if op == "resize_canvas":
        if args.width is None or args.height is None:
            raise SystemExit("--op resize_canvas requires --width and --height")
        return [
            "-background", background_arg(args.background),
            "-gravity", args.gravity,
            "-extent", f"{args.width}x{args.height}",
        ]
    if op == "rotate90_cw":
        return ["-rotate", "90"]
    if op == "rotate90_ccw":
        return ["-rotate", "270"]
    if op == "rotate180":
        return ["-rotate", "180"]
    if op == "flip_horizontal":
        return ["-flop"]
    if op == "flip_vertical":
        return ["-flip"]
    if op == "rotate_arbitrary":
        out: list[str] = []
        if args.filter != "none":
            out += ["-filter", args.filter]
        out += ["-background", background_arg(args.background), "-rotate", str(args.angle)]
        return out
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
        "-write",
        f"{fmt}:{tmp_out}",
        "-format",
        "%wx%h",
        "info:",
    ]
    try:
        run = subprocess.run(cmd, check=True, capture_output=True, text=True)
        result = tmp_out.read_bytes()
    finally:
        tmp_in.unlink(missing_ok=True)
        tmp_out.unlink(missing_ok=True)

    dims = run.stdout.strip().split()[-1] if run.stdout.strip() else ""
    if "x" not in dims:
        print(f"FAIL: could not read output size from magick (stdout={run.stdout!r})", file=sys.stderr)
        return 1
    out_w, out_h = (int(part) for part in dims.split("x"))
    expected_out = out_w * out_h * channels
    if len(result) != expected_out:
        print(
            f"FAIL: ImageMagick wrote {len(result)} bytes, expected {expected_out} "
            f"for {out_w}x{out_h}x{channels}",
            file=sys.stderr,
        )
        return 1
    if args.planar:
        result = planarize(result, out_w, out_h, channels)
    Path(args.output).write_bytes(result)
    print(f"{out_w}x{out_h}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="ImageMagick image-ops oracle for M10-B")
    sub = parser.add_subparsers(dest="command", required=True)

    p_version = sub.add_parser("version", help="print `magick -version`")
    p_version.set_defaults(func=cmd_version)

    p_apply = sub.add_parser("apply", help="apply an ImageMagick operator to a raw image")
    p_apply.add_argument("--size", default="16x16", help="WxH of the raw image")
    p_apply.add_argument("--channels", type=int, choices=(3, 4), default=3)
    p_apply.add_argument("--planar", action="store_true",
                         help="read/write channel planes instead of interleaved")
    p_apply.add_argument("--op", choices=(
        "resize", "resize_canvas", "rotate90_cw", "rotate90_ccw", "rotate180",
        "flip_horizontal", "flip_vertical", "rotate_arbitrary",
    ), help="named operator; builds the Magick args below")
    p_apply.add_argument("--im-args", default="",
                         help="verbatim Magick operator arguments (when --op is omitted)")
    p_apply.add_argument("--width", type=int, default=None, help="output width (resize/canvas)")
    p_apply.add_argument("--height", type=int, default=None, help="output height (resize/canvas)")
    p_apply.add_argument("--filter", default="triangle",
                         help="IM resize/rotate filter (point, triangle, catrom, cubic, none)")
    p_apply.add_argument("--gravity", choices=GRAVITIES, default="center",
                         help="canvas anchor (resize_canvas)")
    p_apply.add_argument("--background", default="0,0,0,255",
                         help="canvas/rotation fill as r,g,b,a (alpha 0..255)")
    p_apply.add_argument("--angle", type=float, default=0.0,
                         help="rotation angle in degrees, positive = clockwise")
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
