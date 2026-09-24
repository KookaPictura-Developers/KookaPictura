#!/usr/bin/env python3
"""ImageMagick compositing oracle for Kooka Pictura task M2-B.

ImageMagick is a second, independent implementation of the blend modes. The
scenes below are deterministic 8x8 RGBA layers; `gen` feeds them through
`magick -compose <operator> -composite` and writes raw interleaved-RGBA8
results under `crates/pictura-render/tests/fixtures/`. The Rust oracle test
rebuilds the same `Document` from the `_base`/`_src` files and diffs
`pictura_render::composite_rgba` against the `_<Mode>` references.

Formats: fixtures are raw interleaved RGBA8 (`r,g,b,a` per pixel, row major),
which the Rust test reads without any codec dependency. `compose`/`gen` route
the layers through PNG internally: reading/writing raw RGBA directly makes
ImageMagick mangle the alpha channel of some operators (notably Difference),
and PNG keeps the alpha semantics correct.

Usage:
    python3 scripts/im_compose.py list [--grep PATTERN]
    python3 scripts/im_compose.py compose MODE BASE SRC OUT [--size 8x8]
    python3 scripts/im_compose.py gen [--out DIR]
    python3 scripts/im_compose.py check

BASE/SRC/OUT may be `.png` or raw `.rgba`. The PSD-mode -> ImageMagick-operator
table lives in MAPPING below and is documented in
`crates/pictura-render/tests/README.md`. `None` means ImageMagick has no
matching operator (or only a deliberately different one) and the mode is not
covered by this oracle.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURE_DIR = ROOT / "crates" / "pictura-render" / "tests" / "fixtures"
MAGICK = os.environ.get("MAGICK", "magick")

WIDTH = HEIGHT = 8
RASTER_SIZE = f"{WIDTH}x{HEIGHT}"

# PSD blend mode -> (ImageMagick compose operator, tolerance, note).
# Tolerance is the absolute per-8-bit-sample difference allowed between the
# Rust compositor and ImageMagick. It is 0 where both use the same formula and
# integer arithmetic, 1 where only rounding can differ, and grows for the
# non-separable HSL modes whose out-of-gamut handling differs.
MAPPING: dict[str, tuple[str | None, int, str]] = {
    # PSD mode:        (IM operator,   tol, note)
    "Normal": ("Over", 0, "Porter-Duff source-over"),
    "Dissolve": (None, 0, "stochastic; IM needs -define compose:args"),
    "Darken": ("Darken", 0, ""),
    "Multiply": ("Multiply", 0, ""),
    "ColorBurn": ("ColorBurn", 0, ""),
    "LinearBurn": ("LinearBurn", 0, ""),
    "DarkerColor": (None, 0, "IM DarkenIntensity compares luminance, not channel sum"),
    "Lighten": ("Lighten", 0, ""),
    "Screen": ("Screen", 0, ""),
    "ColorDodge": ("ColorDodge", 0, ""),
    "LinearDodge": ("LinearDodge", 0, "IM matches min(1, Cb+Cs); keeps Over alpha"),
    "LighterColor": (None, 0, "IM LightenIntensity compares luminance, not channel sum"),
    "Overlay": ("Overlay", 0, ""),
    "SoftLight": (None, 0, "this IM build's SoftLight is not the W3C/PS formula"),
    "HardLight": ("HardLight", 0, ""),
    "VividLight": ("VividLight", 1, "rounding only"),
    "LinearLight": ("LinearLight", 0, ""),
    "PinLight": ("PinLight", 0, ""),
    "HardMix": ("HardMix", 0, "threshold at 0.5; exact-sum boundary differs (scene avoids it)"),
    "Difference": ("Difference", 0, ""),
    "Exclusion": ("Exclusion", 0, ""),
    "Subtract": ("MinusSrc", 0, "PS Subtract = base - source; IM operand order is reversed"),
    "Divide": ("DivideSrc", 0, "PS Divide = base / source; IM operand order is reversed"),
    "Hue": (None, 0, "IM Hue uses HSL-space blending, not SVG non-separable SetSat/SetLum"),
    "Saturation": (None, 0, "IM Saturate uses HSL-space blending, not SVG non-separable"),
    "Color": (None, 0, "IM Colorize uses HSL-space blending, not SVG non-separable"),
    "Luminosity": (None, 0, "IM Luminize uses HSL-space blending, not SVG non-separable"),
}

SUPPORTED = tuple(m for m, (op, _t, _n) in MAPPING.items() if op is not None)


def _pixels(pairs: list[tuple[int, int, int, int]]) -> bytes:
    return bytes(c for p in pairs for c in p)


def scene_solid() -> tuple[bytes, bytes]:
    """Uniform opaque layers: isolates the pure blend function per channel."""
    base = [(64, 128, 192, 255)] * (WIDTH * HEIGHT)
    src = [(200, 100, 50, 255)] * (WIDTH * HEIGHT)
    return _pixels(base), _pixels(src)


def scene_ramp() -> tuple[bytes, bytes]:
    """Opaque color gradients: blend math across many color pairs.

    Both layers stay opaque: ImageMagick's blend operators do not reproduce the
    W3C source-over mix for a semi-transparent source, so partial-alpha coverage
    is exercised by `alpha` (Normal) instead.
    """
    base = []
    src = []
    for y in range(HEIGHT):
        for x in range(WIDTH):
            base.append((10 + 30 * x, 20 + 25 * y, min(255, 40 + 15 * (x + y)), 255))
            src.append((231 - 25 * x, 30 + 25 * y, 180, 255))
    return _pixels(base), _pixels(src)


def scene_alpha() -> tuple[bytes, bytes]:
    """Opaque gradient base under a source with a varying alpha ramp."""
    base = []
    src = []
    for y in range(HEIGHT):
        for x in range(WIDTH):
            base.append((10 + 30 * x, 20 + 25 * y, min(255, 40 + 15 * (x + y)), 255))
            src.append((220, 90, 40, max(0, 255 - 32 * x)))
    return _pixels(base), _pixels(src)


# scene -> (pixel builder, PSD modes to generate references for).
SCENES: dict[str, tuple[Callable[[], tuple[bytes, bytes]], tuple[str, ...]]] = {
    "solid": (scene_solid, SUPPORTED),
    "ramp": (scene_ramp, SUPPORTED),
    "alpha": (scene_alpha, ("Normal",)),
}


def im_available() -> bool:
    return shutil.which(MAGICK) is not None


def _as_png(path: Path, tmp: Path) -> Path:
    if path.suffix == ".png":
        return path
    png = tmp / (path.stem + ".png")
    subprocess.run(
        [MAGICK, "-size", RASTER_SIZE, "-depth", "8", f"rgba:{path}", str(png)],
        check=True,
        capture_output=True,
        text=True,
    )
    return png


def im_compose(mode: str, base: Path, src: Path, out: Path, tmp: Path) -> None:
    """Composite `src` over `base` with `mode`; `out` is raw RGBA8 or PNG."""
    cmd = [MAGICK, str(_as_png(base, tmp)), str(_as_png(src, tmp)),
           "-compose", mode, "-composite"]
    if out.suffix == ".png":
        cmd.append(str(out))
    else:
        cmd += ["-depth", "8", f"rgba:{out}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)


def generate(out_dir: Path) -> list[Path]:
    out_dir.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    with tempfile.TemporaryDirectory() as tmp_name:
        tmp = Path(tmp_name)
        for scene, (build, modes) in SCENES.items():
            base_px, src_px = build()
            base = out_dir / f"{scene}_base.rgba"
            src = out_dir / f"{scene}_src.rgba"
            base.write_bytes(base_px)
            src.write_bytes(src_px)
            written += [base, src]
            for ps_mode in modes:
                operator = MAPPING[ps_mode][0]
                assert operator is not None
                out = out_dir / f"{scene}_{ps_mode}.rgba"
                im_compose(operator, base, src, out, tmp)
                written.append(out)
    return written


def cmd_list(args: argparse.Namespace) -> int:
    result = subprocess.run(
        [MAGICK, "-list", "compose"], check=True, capture_output=True, text=True
    )
    lines = result.stdout.splitlines()
    if args.grep:
        needle = args.grep.lower()
        lines = [ln for ln in lines if needle in ln.lower()]
    print("\n".join(lines))
    return 0


def cmd_compose(args: argparse.Namespace) -> int:
    with tempfile.TemporaryDirectory() as tmp_name:
        im_compose(
            args.mode, Path(args.base), Path(args.src), Path(args.out), Path(tmp_name)
        )
    print(f"wrote {args.out}")
    return 0


def _fixture_bytes(directory: Path) -> dict[str, bytes]:
    return {p.name: p.read_bytes() for p in sorted(directory.glob("*.rgba"))}


def cmd_check(_args: argparse.Namespace) -> int:
    committed = _fixture_bytes(FIXTURE_DIR)
    if not committed:
        print(f"FAIL: no fixtures under {FIXTURE_DIR}; run `gen`", file=sys.stderr)
        return 1
    with tempfile.TemporaryDirectory() as tmp_name:
        generate(Path(tmp_name))
        fresh = _fixture_bytes(Path(tmp_name))
    if committed.keys() != fresh.keys():
        missing = fresh.keys() - committed.keys()
        extra = committed.keys() - fresh.keys()
        if missing:
            print(f"FAIL: missing fixtures {sorted(missing)}; run `gen`", file=sys.stderr)
            return 1
        # Non-ImageMagick references (for example the psd-tools knockout oracle)
        # share the directory; only the files this script generates are checked.
        print(f"note: ignoring {len(extra)} non-ImageMagick fixture(s): {sorted(extra)}")
    for name in fresh:
        if committed[name] != fresh[name]:
            print(f"FAIL: {name} not reproducible (bytes differ)", file=sys.stderr)
            return 1
    print(f"OK: {len(fresh)} fixtures reproduce byte-for-byte")
    return 0


def cmd_gen(args: argparse.Namespace) -> int:
    out = Path(args.out) if args.out else FIXTURE_DIR
    written = generate(out)
    print(f"wrote {len(written)} fixtures under {out}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="ImageMagick compositing oracle for M2-B")
    sub = parser.add_subparsers(dest="command", required=True)

    p_list = sub.add_parser("list", help="print `magick -list compose`")
    p_list.add_argument("--grep", help="case-insensitive substring filter")
    p_list.set_defaults(func=cmd_list)

    p_compose = sub.add_parser("compose", help="composite two RGBA layers")
    p_compose.add_argument("mode", help="ImageMagick compose operator")
    p_compose.add_argument("base")
    p_compose.add_argument("src")
    p_compose.add_argument("out")
    p_compose.set_defaults(func=cmd_compose)

    p_gen = sub.add_parser("gen", help="regenerate the committed fixtures")
    p_gen.add_argument("--out", help="output directory (default: tests/fixtures)")
    p_gen.set_defaults(func=cmd_gen)

    p_check = sub.add_parser("check", help="verify fixtures reproduce byte-for-byte")
    p_check.set_defaults(func=cmd_check)

    args = parser.parse_args(argv)
    if not im_available():
        print(f"FAIL: '{MAGICK}' not found on PATH (set MAGICK to override)", file=sys.stderr)
        return 1
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
