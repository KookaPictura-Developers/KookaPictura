#!/usr/bin/env python3
"""Vendor Lucide SVGs into assets/icons/ per assets/icons/lucide-map.json.

For every entry whose kind is `lucide`, copies the pinned release's
`icons/<slug>.svg` to `assets/icons/<id>.svg`. Entries of kind `custom`,
`note`, or `out-of-scope` are left untouched (custom art is checked in and
hand-authored).

Usage:
    python3 scripts/sync-lucide-icons.py --source /path/to/lucide-static/icons
    python3 scripts/sync-lucide-icons.py --source DIR --check   # dry run
"""
from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ICONS = ROOT / "assets" / "icons"
MAP = ICONS / "lucide-map.json"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", required=True,
                    help="directory holding the Lucide release's icon SVG files")
    ap.add_argument("--check", action="store_true",
                    help="report what would change without writing")
    args = ap.parse_args()

    source = Path(args.source)
    if not source.is_dir():
        print(f"source directory not found: {source}")
        return 2

    doc = json.loads(MAP.read_text(encoding="utf-8"))
    entries: dict[str, dict] = doc["entries"]

    copied = 0
    skipped = 0
    missing: list[str] = []
    for pid, rec in sorted(entries.items()):
        if rec.get("kind") != "lucide":
            skipped += 1
            continue
        slug = rec["slug"]
        src = source / f"{slug}.svg"
        if not src.is_file():
            missing.append(f"{pid} -> {slug}")
            continue
        if args.check:
            print(f"would copy {slug}.svg -> {pid}.svg")
        else:
            shutil.copyfile(src, ICONS / f"{pid}.svg")
        copied += 1

    for item in missing:
        print(f"MISSING lucide icon: {item}", file=sys.stderr)
    verb = "would copy" if args.check else "copied"
    print(f"{verb}={copied} left-as-is={skipped} missing={len(missing)}")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
