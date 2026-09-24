#!/usr/bin/env python3
"""psd-tools knockout pixel reference for `pictura-render`.

`psd-tools` ships an independent compositor that implements deep knockout
(`psd_tools/composite/composite.py`, verified against Photoshop). It is the
second implementation for the CPU knockout path, exactly as ImageMagick is for
the blend modes and psd-tools itself is for the codec.

`gen` opens the committed `knockout.psd`, composites it, converts to RGBA, and
writes raw interleaved RGBA8 to
`crates/pictura-render/tests/fixtures/knockout_deep.rgba`, which
`tests/knockout_oracle.rs` reads without a codec dependency.

Formats: raw interleaved RGBA8 (`r,g,b,a` per pixel, row major), 8x8.

Usage:
    python3 scripts/psd_knockout_reference.py gen [--out PATH]
"""

from __future__ import annotations

import argparse
from pathlib import Path

from psd_tools import PSDImage

ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "crates" / "pictura-codec" / "tests" / "fixtures" / "knockout.psd"
DEFAULT_OUT = ROOT / "crates" / "pictura-render" / "tests" / "fixtures" / "knockout_deep.rgba"


def gen(out: Path) -> None:
    image = PSDImage.open(FIXTURE).composite().convert("RGBA")
    out.write_bytes(image.tobytes())
    print(f"wrote {out.relative_to(ROOT)} ({image.width}x{image.height})")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="psd-tools knockout pixel reference")
    sub = parser.add_subparsers(dest="command", required=True)

    p_gen = sub.add_parser("gen", help="regenerate the committed reference")
    p_gen.add_argument("--out", default=str(DEFAULT_OUT))
    p_gen.set_defaults(func=lambda args: gen(Path(args.out)))

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
