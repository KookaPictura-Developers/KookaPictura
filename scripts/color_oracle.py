#!/usr/bin/env python3
"""ImageMagick ICC conversion oracle for Kooka Pictura task M3-B.

ImageMagick links Little CMS 2, so `magick ... -profile src -profile dst` is an
independent check of `pictura_color::convert`'s plumbing (channel order,
stride, bit depth, intent/BPC flags) rather than a re-implementation of the
color transform.

`convert` reads interleaved RGBA8 and runs:

    magick -size WxH -depth 8 rgba:IN \
        -intent <intent> [-black-point-compensation] \
        -profile SRC.icc -profile DST.icc \
        -depth 8 rgba:OUT

The raw input is untagged, so the first `-profile` attaches SRC and the second
transforms it to DST using the `-intent`/BPC settings that precede it.
`-intent`/`-black-point-compensation` are settings, applied when the second
`-profile` runs.

Usage:
    python3 scripts/color_oracle.py version
    python3 scripts/color_oracle.py convert --src SRC.icc --dst DST.icc \
        [--intent relative|perceptual|saturation|absolute|undefined] [--bpc] \
        [--size 8x8] IN.rgba OUT.rgba

Flags, tolerances and the measured divergence between ImageMagick and lcms2 are
documented in `crates/pictura-color/tests/README.md`.

Only 8-bit RGBA is handled: ImageMagick's raw coder writes 16-bit samples in
big-endian order, which the M3 contract does not exercise yet.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

MAGICK = os.environ.get("MAGICK", "magick")
INTENTS = ("relative", "perceptual", "saturation", "absolute", "undefined")


def im_available() -> bool:
    return shutil.which(MAGICK) is not None


def magick_command(src: Path, dst: Path, in_path: Path, out_path: Path,
                   size: str, intent: str, bpc: bool) -> list[str]:
    cmd = [MAGICK, "-size", size, "-depth", "8", f"rgba:{in_path}"]
    if intent != "undefined":
        cmd += ["-intent", intent]
    if bpc:
        cmd.append("-black-point-compensation")
    cmd += ["-profile", str(src), "-profile", str(dst),
            "-depth", "8", f"rgba:{out_path}"]
    return cmd


def cmd_version(_args: argparse.Namespace) -> int:
    result = subprocess.run([MAGICK, "-version"], check=True, capture_output=True, text=True)
    print(result.stdout, end="")
    return 0


def cmd_convert(args: argparse.Namespace) -> int:
    size = args.size
    if "x" not in size:
        print(f"FAIL: --size must be WxH, got {size!r}", file=sys.stderr)
        return 1
    cmd = magick_command(Path(args.src), Path(args.dst), Path(args.input),
                         Path(args.output), size, args.intent, args.bpc)
    subprocess.run(cmd, check=True, capture_output=True, text=True)
    print(f"wrote {args.output}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="ImageMagick ICC conversion oracle for M3-B")
    sub = parser.add_subparsers(dest="command", required=True)

    p_version = sub.add_parser("version", help="print `magick -version` (incl. lcms)")
    p_version.set_defaults(func=cmd_version)

    p_convert = sub.add_parser("convert", help="convert raw RGBA8 via ICC profiles")
    p_convert.add_argument("--src", required=True, help="source ICC profile (attached)")
    p_convert.add_argument("--dst", required=True, help="destination ICC profile")
    p_convert.add_argument("--intent", choices=INTENTS, default="relative",
                           help="rendering intent; 'undefined' omits -intent")
    p_convert.add_argument("--bpc", action="store_true",
                           help="enable black point compensation")
    p_convert.add_argument("--size", default="8x8", help="WxH of the raw image")
    p_convert.add_argument("input", help="raw interleaved RGBA8 input")
    p_convert.add_argument("output", help="raw interleaved RGBA8 output")
    p_convert.set_defaults(func=cmd_convert)

    args = parser.parse_args(argv)
    if not im_available():
        print(f"FAIL: '{MAGICK}' not found on PATH (set MAGICK to override)",
              file=sys.stderr)
        return 1
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
