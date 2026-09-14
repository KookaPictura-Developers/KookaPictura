#!/usr/bin/env python3
"""ImageMagick adjustment oracle for Kooka Pictura task M4-C.

ImageMagick is an independent implementation of several `Image > Adjustments`
operators. This script applies one of them to a raw 8-bit image and emits the
result so `crates/pictura-adjust/tests/oracle.rs` can diff it against
`pictura_adjust::apply`. It is a *sanity* oracle, not a parity oracle: Adobe's
exact integer math is closed, and ImageMagick's operators only approximate some
of the Photoshop adjustments.

Layout: raw 8-bit samples, interleaved `rgb`/`rgba` by default. `--planar`
reads/writes channel planes (`RRR…GGG…BBB…`, matching `PixelBuffer::data`).

Operators are passed through to ImageMagick verbatim via `--im-args`; the named
`--op` (plus flags) is a convenience that builds the same argument list. The
adjustment -> ImageMagick mapping and the measured/tolerated divergence live in
`crates/pictura-adjust/tests/README.md`; the authoritative Magick flags are:

    Adjustment          ImageMagick
    ------------------  -----------------------------------------------------
    Levels              -level B%,W%,gamma  +level Ob%,Ow%
    Gamma (part of Lv)  -gamma g
    Brightness/Contrast -brightness-contrast BxC
    Invert              -negate
    Posterize           -posterize n
    Threshold           -threshold T%
    Desaturate          -modulate 100,0,100        (== (min+max)/2)
    Hue/Saturation      -modulate 100,S,H          (Master only)
    Channel Mixer       -color-matrix "m0,…,m8"
    Exposure            -evaluate multiply|add|pow  (encoded space, NOT linear)

Percentages are required for `-level`, `-threshold`, `+level` and additive
`-evaluate` because a Q16 build otherwise reads bare numbers as 0…65535.

Usage:
    python3 scripts/adjust_oracle.py version
    python3 scripts/adjust_oracle.py apply --size 8x8 --planar \
        --im-args="-negate" IN.rgb OUT.rgb
    python3 scripts/adjust_oracle.py apply --size 8x8 --op levels \
        --black 0 --white 255 --gamma 2.0 --out-black 0 --out-white 255 \
        IN.rgb OUT.rgb

`--im-args` must be attached with `=` (`--im-args="-level 0%,100%,2.0"`) or
argparse mistakes the leading `-` for another option.
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


def pct(value: float, maximum: float = 255.0) -> str:
    """8-bit sample -> ImageMagick percentage (Q16 builds need the `%`)."""
    return f"{value / maximum * 100.0:.6f}%"


def build_im_args(args: argparse.Namespace) -> list[str]:
    """Translate a named --op plus flags into Magick operator arguments."""
    op = args.op
    if op is None:
        return shlex.split(args.im_args)

    if op == "levels":
        return [
            "-level",
            f"{pct(args.black)},{pct(args.white)},{args.gamma}",
            "+level",
            f"{pct(args.out_black)},{pct(args.out_white)}",
        ]
    if op == "gamma":
        return ["-gamma", str(args.value)]
    if op == "brightness-contrast":
        return ["-brightness-contrast", f"{args.brightness}x{args.contrast}"]
    if op == "negate":
        return ["-negate"]
    if op == "posterize":
        return ["-posterize", str(args.levels)]
    if op == "threshold":
        return ["-threshold", pct(args.level)]
    if op == "gray":
        return ["-colorspace", "Gray"]
    if op == "desaturate":
        return ["-modulate", "100,0,100"]
    if op == "modulate":
        return ["-modulate", f"{args.brightness},{args.saturation},{args.hue}"]
    if op == "color-matrix":
        return ["-color-matrix", args.matrix]
    if op == "evaluate":
        if args.eval_op is None or args.eval_value is None:
            raise SystemExit("evaluate requires --eval-op and --eval-value")
        return ["-evaluate", args.eval_op, args.eval_value]
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
    parser = argparse.ArgumentParser(description="ImageMagick adjustment oracle for M4-C")
    sub = parser.add_subparsers(dest="command", required=True)

    p_version = sub.add_parser("version", help="print `magick -version`")
    p_version.set_defaults(func=cmd_version)

    p_apply = sub.add_parser("apply", help="apply an ImageMagick operator to a raw image")
    p_apply.add_argument("--size", default="8x8", help="WxH of the raw image")
    p_apply.add_argument("--channels", type=int, choices=(3, 4), default=3)
    p_apply.add_argument("--planar", action="store_true",
                         help="read/write channel planes instead of interleaved")
    p_apply.add_argument("--op", choices=(
        "levels", "gamma", "brightness-contrast", "negate", "posterize",
        "threshold", "gray", "desaturate", "modulate", "color-matrix", "evaluate",
    ), help="named operator; builds the Magick args below")
    p_apply.add_argument("--im-args", default="",
                         help="verbatim Magick operator arguments (when --op is omitted)")
    p_apply.add_argument("--black", type=float, default=0)
    p_apply.add_argument("--white", type=float, default=255)
    p_apply.add_argument("--gamma", type=float, default=1.0)
    p_apply.add_argument("--out-black", type=float, default=0)
    p_apply.add_argument("--out-white", type=float, default=255)
    p_apply.add_argument("--value", type=float, default=1.0)
    p_apply.add_argument("--brightness", type=float, default=100)
    p_apply.add_argument("--contrast", type=float, default=0)
    p_apply.add_argument("--levels", type=int, default=4)
    p_apply.add_argument("--level", type=float, default=128)
    p_apply.add_argument("--saturation", type=float, default=100)
    p_apply.add_argument("--hue", type=float, default=100)
    p_apply.add_argument("--matrix", default="1,0,0,0,1,0,0,0,1")
    p_apply.add_argument("--eval-op", dest="eval_op")
    p_apply.add_argument("--eval-value", dest="eval_value")
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
