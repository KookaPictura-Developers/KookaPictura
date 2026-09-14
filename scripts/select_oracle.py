#!/usr/bin/env python3
"""ImageMagick morphology/blur oracle for Kooka Pictura task M5-B.

ImageMagick is an independent implementation of grayscale morphology and
Gaussian blur. This script applies one operator to a raw 8-bit grayscale mask
(one byte per pixel, the selection coverage layout) and emits the result so
`crates/pictura-select/tests/oracle.rs` can diff it against the `Selection`
modify ops. It is a *sanity* oracle, not a parity oracle: Adobe does not publish
the structuring element (square vs. disk) or the feather radius->sigma mapping,
so only the shape/magnitude of the operation is checked.

The selection -> ImageMagick mapping (authoritative flags in
`crates/pictura-select/tests/README.md`):

    Selection::expand(r)    -morphology Dilate Square:r     (separable box)
    Selection::contract(r)  -morphology Erode  Square:r      (separable box)
    Selection::smooth(r)    -morphology Smooth Square:r      (IM mean, NOT PS majority)
    Selection::feather(r)   -gaussian-blur 0x(r/2)           (sigma = r/2, matches M5-A)
    invert                  -negate
    (threshold)             -threshold T%

`pictura_select` (M5-A) implements expand/contract as a **separable box**
min/max over a `(2r+1)x(2r+1)` window, which is `-morphology ... Square:r`.
The Euclidean `Disk:r` (`--radius r`, the default when `--kernel` is omitted)
is the structuring element the spec *proposed*; it differs at the diagonal
corners (4 pixels, max delta 255 on the 8x8 test mask), so the differential
tests pass `--kernel Square:r` explicitly.

ImageMagick morphology details that the tests depend on:

  * ImageMagick's default `-virtual-pixel` is `edge`, so a selection touching
    the canvas border is **not** eroded from that border. This matches
    Photoshop's documented canvas-edge exemption and M5-A's edge clamp.
  * Q16-HDRI builds compute at 16-bit (float) precision and round to 8 bits on
    output; a binary mask survives morphology exactly (min/max), so the
    differential tolerance for expand/contract is 0.

Usage:
    python3 scripts/select_oracle.py version
    python3 scripts/select_oracle.py apply --size 8x8 --op dilate --radius 1 \
        IN.gray OUT.gray
    python3 scripts/select_oracle.py apply --size 8x8 --op erode --kernel Square:1 \
        IN.gray OUT.gray
    python3 scripts/select_oracle.py apply --size 8x8 --op gaussian --sigma 2.0 \
        IN.gray OUT.gray
    python3 scripts/select_oracle.py apply --size 8x8 --im-args="-negate" \
        IN.gray OUT.gray

`--im-args` must be attached with `=` (`--im-args="-morphology Dilate Disk:2"`)
or argparse mistakes the leading `-` for another option.
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

MORPHOLOGY = ("dilate", "erode", "smooth", "open", "close", "gradient")


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

    if op in MORPHOLOGY:
        kernel = args.kernel or f"Disk:{args.radius}"
        return ["-morphology", op.capitalize(), kernel]
    if op == "blur":
        return ["-blur", f"0x{args.sigma}"]
    if op == "gaussian":
        return ["-gaussian-blur", f"0x{args.sigma}"]
    if op == "negate":
        return ["-negate"]
    if op == "threshold":
        return ["-threshold", pct(args.level)]
    raise SystemExit(f"unknown --op {op!r}")


def cmd_version(_args: argparse.Namespace) -> int:
    result = subprocess.run([MAGICK, "-version"], check=True, capture_output=True, text=True)
    print(result.stdout, end="")
    return 0


def cmd_apply(args: argparse.Namespace) -> int:
    if "x" not in args.size:
        print(f"FAIL: --size must be WxH, got {args.size!r}", file=sys.stderr)
        return 1
    width, height = (int(part) for part in args.size.split("x"))
    expected = width * height

    raw = Path(args.input).read_bytes()
    if len(raw) != expected:
        print(
            f"FAIL: {args.input} has {len(raw)} bytes, expected {expected} "
            f"for {width}x{height} grayscale",
            file=sys.stderr,
        )
        return 1

    im_args = build_im_args(args)
    tmp_in = Path(args.input).with_suffix(".oracle-in.gray")
    tmp_out = Path(args.output).with_suffix(".oracle-out.gray")
    tmp_in.write_bytes(raw)
    cmd = [
        MAGICK,
        "-size",
        args.size,
        "-depth",
        "8",
        f"gray:{tmp_in}",
        *im_args,
        "-depth",
        "8",
        f"gray:{tmp_out}",
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
    Path(args.output).write_bytes(result)
    print(f"wrote {args.output}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="ImageMagick selection-mask oracle for M5-B")
    sub = parser.add_subparsers(dest="command", required=True)

    p_version = sub.add_parser("version", help="print `magick -version`")
    p_version.set_defaults(func=cmd_version)

    p_apply = sub.add_parser("apply", help="apply an ImageMagick operator to a raw gray mask")
    p_apply.add_argument("--size", default="8x8", help="WxH of the raw mask")
    p_apply.add_argument("--op", choices=(
        *MORPHOLOGY, "blur", "gaussian", "negate", "threshold",
    ), help="named operator; builds the Magick args below")
    p_apply.add_argument("--im-args", default="",
                         help="verbatim Magick operator arguments (when --op is omitted)")
    p_apply.add_argument("--radius", type=int, default=1,
                         help="morphology radius -> Disk:r (default 1)")
    p_apply.add_argument("--kernel", default=None,
                         help="override the structuring element, e.g. Square:1")
    p_apply.add_argument("--sigma", type=float, default=1.0,
                         help="Gaussian sigma for --op blur/gaussian")
    p_apply.add_argument("--level", type=float, default=128,
                         help="threshold level (0..255) for --op threshold")
    p_apply.add_argument("input", help="raw 8-bit grayscale mask")
    p_apply.add_argument("output", help="raw 8-bit grayscale mask")
    p_apply.set_defaults(func=cmd_apply)

    args = parser.parse_args(argv)
    if not im_available():
        print(f"FAIL: '{MAGICK}' not found on PATH (set MAGICK to override)",
              file=sys.stderr)
        return 1
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
