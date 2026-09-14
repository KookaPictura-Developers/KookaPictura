#!/usr/bin/env python3
"""Validate that a PSD written by `pictura-codec` opens in the independent
`psd-tools` library, and print its layer tree.

Usage:  python3 scripts/validate_output.py path/to/output.psd

Prints each layer's name, kind, bounds, size, and blend mode. Exits non-zero
if the file cannot be opened or parsed.
"""

from __future__ import annotations

import argparse
import sys

from psd_tools import PSDImage


def _dump(layer, indent: int = 0) -> None:
    kind = "group" if layer.is_group() else "pixel"
    width, height = layer.size
    visible = "" if layer.visible else " hidden"
    mask = " +mask" if layer.has_mask() else ""
    print(
        f"{'  ' * indent}- {layer.name!r} [{kind}] "
        f"bounds={layer.bbox} size={width}x{height} "
        f"mode={layer.blend_mode.name}{visible}{mask}"
    )
    if layer.is_group():
        for child in layer:
            _dump(child, indent + 1)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", help="PSD file to open")
    args = parser.parse_args()

    try:
        psd = PSDImage.open(args.path)
    except Exception as error:  # noqa: BLE001 - report any parse failure
        print(f"FAIL: cannot open {args.path}: {error}", file=sys.stderr)
        return 1

    print(
        f"{args.path}: {psd.width}x{psd.height} "
        f"mode={psd.color_mode.name} depth={psd.depth} layers={len(psd)}"
    )
    for layer in psd:
        _dump(layer)
    return 0


if __name__ == "__main__":
    sys.exit(main())
