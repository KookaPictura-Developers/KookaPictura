#!/usr/bin/env python3
"""Unified Kooka Pictura test report (python3 standard library only).

Aggregates three layers into one pytest/vitest-like report:

  * nextest JUnit XML      (--junit)
  * cargo test --doc text  (--doctests)
  * C++ self-test tokens   (--selftest)

Exits non-zero when any layer failed or a provided input could not be parsed.
Run ``python3 scripts/report_tests.py --self-check`` for the parser fixture
checks.
"""

import argparse
import os
import re
import sys
import xml.etree.ElementTree as ET

RULE = "\u2500" * 48
TOKEN_PREFIX_RE = re.compile(
    r"^pictura self-test: (SUITE|PASS|SKIP|FAIL|SUMMARY)(?:\s+(.*))?$"
)
DOCTEST_RESULT_RE = re.compile(
    r"test result: (?:ok|FAILED)\.\s+(\d+) passed;\s+(\d+) failed"
    r"(?:;\s+(\d+) ignored)?"
)
DOCTEST_HEADER_RE = re.compile(r"^---- (.+?) stdout ----$", re.MULTILINE)


class Counts:
    __slots__ = ("passed", "failed", "skipped")

    def __init__(self, passed=0, failed=0, skipped=0):
        self.passed = passed
        self.failed = failed
        self.skipped = skipped

    def add(self, other):
        self.passed += other.passed
        self.failed += other.failed
        self.skipped += other.skipped

    def total(self):
        return self.passed + self.failed + self.skipped


def sum_counts(suites):
    total = Counts()
    for counts in suites.values():
        total.add(counts)
    return total


def _node_message(node):
    if node is None:
        return ""
    text = (node.text or "").strip()
    if text:
        return text
    bits = [node.get("message"), node.get("type")]
    return " ".join(bit for bit in bits if bit).strip()


def parse_junit(xml_text):
    """Return ({package: Counts}, [(test_id, message)])."""
    root = ET.fromstring(xml_text)
    suites = {}
    failures = []
    for testsuite in root.iter("testsuite"):
        name = testsuite.get("name") or ""
        package = name.split("::", 1)[0] or name
        counts = suites.setdefault(package, Counts())
        cases = testsuite.findall("testcase")
        if cases:
            for case in cases:
                bad = case.find("failure")
                if bad is None:
                    bad = case.find("error")
                if bad is not None:
                    counts.failed += 1
                    container = case.get("classname") or name
                    tid = f"{container}::{case.get('name') or '?'}"
                    failures.append((tid, _node_message(bad)))
                elif case.find("skipped") is not None:
                    counts.skipped += 1
                else:
                    counts.passed += 1
        else:
            tests = int(testsuite.get("tests") or 0)
            failed = int(testsuite.get("failures") or 0) + int(
                testsuite.get("errors") or 0
            )
            skipped = int(testsuite.get("disabled") or testsuite.get("skipped") or 0)
            counts.passed += max(tests - failed - skipped, 0)
            counts.failed += failed
            counts.skipped += skipped
    return suites, failures


def parse_doctests(text):
    """Return (Counts, [(test_id, message)], summary_seen)."""
    counts = Counts()
    seen = False
    for match in DOCTEST_RESULT_RE.finditer(text):
        seen = True
        counts.passed += int(match.group(1))
        counts.failed += int(match.group(2))
        counts.skipped += int(match.group(3) or 0)

    failures = []
    headers = list(DOCTEST_HEADER_RE.finditer(text))
    for index, header in enumerate(headers):
        end = headers[index + 1].start() if index + 1 < len(headers) else len(text)
        body = text[header.end():end]
        message = "\n".join(line for line in body.splitlines() if line.strip())
        failures.append((f"doctests::{header.group(1)}", message[:800]))
    if counts.failed and not headers:
        failures.append(("doctests", "one or more doctests failed"))
    return counts, failures, seen


def parse_selftest_stream(text):
    """Parse one self-test stream into (order, checks, summary).

    Only the machine tokens count. Human ``key=value`` progress lines and
    ``FAIL:`` lines do not match the token keyword grammar. ``order`` lists
    ``(suite, name)`` keys first-seen; ``checks`` maps each key to its latest
    record; ``summary`` is the Counts from a SUMMARY line, or None.
    """
    order = []
    checks = {}
    summary = None
    for raw in text.splitlines():
        match = TOKEN_PREFIX_RE.match(raw.strip())
        if not match:
            continue
        keyword = match.group(1)
        rest = (match.group(2) or "").strip()
        if keyword == "SUMMARY":
            found = dict(re.findall(r"(passed|failed|skipped)=(\d+)", rest))
            if found:
                summary = Counts(
                    int(found.get("passed", 0)),
                    int(found.get("failed", 0)),
                    int(found.get("skipped", 0)),
                )
            continue
        if keyword == "SUITE":
            continue
        parts = rest.split(None, 2)
        if len(parts) < 2:
            continue
        key = (parts[0], parts[1])
        record = {"status": keyword, "code": "", "message": ""}
        if keyword == "FAIL" and len(parts) == 3:
            tail = parts[2].split(None, 1)
            record["code"] = tail[0]
            record["message"] = tail[1] if len(tail) == 2 else ""
        if key not in checks:
            order.append(key)
        checks[key] = record
    return order, checks, summary


def merge_selftest_streams(texts):
    """Merge self-test streams, deduping checks by (suite, name).

    A check present in several runs is counted once; the last stream's status
    wins. Layer totals come from the SUMMARY only when a single stream is
    supplied, otherwise they are recomputed from the merged unique checks.
    Returns ({suite: Counts}, totals, [(label, message)]).
    """
    order = []
    merged = {}
    summaries = []
    for text in texts:
        stream_order, checks, summary = parse_selftest_stream(text)
        summaries.append(summary)
        for key in stream_order:
            if key not in merged:
                order.append(key)
            merged[key] = checks[key]

    suites = {}
    failures = []
    for key in order:
        suite, name = key
        record = merged[key]
        counts = suites.setdefault(suite, Counts())
        status = record["status"]
        if status == "PASS":
            counts.passed += 1
        elif status == "SKIP":
            counts.skipped += 1
        else:
            counts.failed += 1
            label = f"{suite}::{name}"
            if record["code"]:
                label += f" (exit {record['code']})"
            failures.append((label, record["message"]))

    if len(texts) == 1 and summaries[0] is not None:
        totals = summaries[0]
    else:
        totals = sum_counts(suites)
    return suites, totals, failures


def parse_selftest(text):
    """Return ({suite: Counts}, totals, failures) for a single stream."""
    return merge_selftest_streams([text])


def _counts_compact(counts):
    parts = [f"{counts.passed} passed"]
    if counts.skipped:
        parts.append(f"{counts.skipped} skipped")
    if counts.failed:
        parts.append(f"{counts.failed} failed")
    return ", ".join(parts)


def _counts_dot(counts):
    return f"{counts.passed} passed \u00b7 {counts.skipped} skipped \u00b7 {counts.failed} failed"


def _style(text, code, color):
    if not color:
        return text
    return f"\x1b[{code}m{text}\x1b[0m"


class Layer:
    def __init__(self, title, suites=None, totals=None, single=False):
        self.title = title
        self.suites = suites or {}
        self.totals = totals if totals is not None else sum_counts(self.suites)
        self.single = single

    def empty(self):
        return self.totals.total() == 0 and not self.suites


def render(layers, failures, color):
    visible = [layer for layer in layers if not layer.empty()]
    labels = []
    for layer in visible:
        labels.extend(layer.suites)
        labels.append("subtotal")
        if layer.single:
            labels.append(layer.title)
    width = max([24] + [len(label) for label in labels])

    lines = [_style("pictura test report", "1", color), RULE]
    totals = Counts()
    for layer in visible:
        totals.add(layer.totals)
        if layer.single:
            lines.append(
                f"{layer.title:<{width + 2}}{_counts_compact(layer.totals)}"
            )
            continue
        lines.append(_style(layer.title, "1", color))
        for name, counts in layer.suites.items():
            lines.append(f"  {name:<{width}}{_counts_compact(counts)}")
        lines.append(f"  {'subtotal':<{width}}{_counts_dot(layer.totals)}")
    lines.append(RULE)
    lines.append(_style(f"TOTAL  {_counts_dot(totals)}", "1", color))

    if failures:
        lines.append("")
        lines.append(_style("FAILURES", "1;31", color))
        for index, (label, message) in enumerate(failures, 1):
            lines.append(f"  {index}) {label}")
            body = message if message.strip() else "(no message)"
            for line in body.splitlines():
                lines.append(f"       {line}")
    return "\n".join(lines)


def _read(path, errors):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError as exc:
        errors.append(f"{path}: {exc.strerror or exc}")
        return None


def build_layers(args, errors):
    layers = []
    failures = []

    if args.junit:
        text = _read(args.junit, errors)
        if text is not None:
            try:
                suites, junit_failures = parse_junit(text)
            except ET.ParseError as exc:
                errors.append(f"{args.junit}: invalid JUnit XML: {exc}")
            else:
                layers.append(Layer("Rust suites (nextest)", suites))
                failures.extend(junit_failures)

    if args.selftest:
        texts = []
        for path in args.selftest:
            text = _read(path, errors)
            if text is not None:
                texts.append(text)
        if texts:
            suites, totals, selftest_failures = merge_selftest_streams(texts)
            if suites or totals.total():
                layers.append(
                    Layer("App self-test (offscreen Qt)", suites, totals)
                )
            failures.extend(selftest_failures)

    if args.doctests:
        text = _read(args.doctests, errors)
        if text is not None:
            counts, doctest_failures, seen = parse_doctests(text)
            if not seen:
                errors.append(
                    f"{args.doctests}: no 'test result:' summary line found"
                )
            elif counts.total():
                layers.append(Layer("Doctests", totals=counts, single=True))
            failures.extend(doctest_failures)

    return layers, failures


def exit_code(failures, errors):
    return 1 if failures or errors else 0


def _color_enabled(mode):
    if mode == "always":
        return True
    if mode == "never":
        return False
    return sys.stdout.isatty()


def _parse_args(argv):
    parser = argparse.ArgumentParser(description="Unified Kooka Pictura test report")
    parser.add_argument("--junit", help="nextest JUnit XML path")
    parser.add_argument("--doctests", help="cargo test --doc output path")
    parser.add_argument(
        "--selftest",
        action="append",
        help="captured self-test output path (repeatable; streams are merged)",
    )
    parser.add_argument(
        "--color", choices=("auto", "always", "never"), default="auto"
    )
    parser.add_argument("--no-color", action="store_true")
    parser.add_argument("--self-check", action="store_true")
    return parser.parse_args(argv)


PASS_JUNIT = """<testsuites>
  <testsuite name="pictura-core::unittests" tests="2">
    <testcase name="a" classname="pictura-core"/>
    <testcase name="b" classname="pictura-core"/>
  </testsuite>
  <testsuite name="pictura-codec::oracle" tests="1">
    <testcase name="oracle::x" classname="pictura-codec"/>
  </testsuite>
</testsuites>"""

FAIL_JUNIT = """<testsuites>
  <testsuite name="pictura-codec::oracle" tests="2" failures="1">
    <testcase name="oracle::skipped" classname="pictura-codec"><skipped/></testcase>
    <testcase name="oracle::bad" classname="pictura-codec">
      <failure message="boom">assertion `left == right` failed</failure>
    </testcase>
  </testsuite>
</testsuites>"""

DOCTEST_OK = (
    "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; "
    "0 filtered out; finished in 0.01s\n"
)

DOCTEST_FAIL = """test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out

failures:

---- foo::bar (line 3) stdout ----
Error: assertion failed
"""

SELFTEST = """pictura self-test: document_size=4000
pictura self-test: SUITE core document_size
pictura self-test: PASS core document_size 4000x4000
pictura self-test: SKIP core gpu vk not available
pictura self-test: FAIL: human failure line
pictura self-test: FAIL m47 compact_shade 195 M47 compact shade
pictura self-test: SUMMARY passed=1 failed=1 skipped=1
"""

SELFTEST_BARE = """pictura self-test: PASS core shared token from bare
pictura self-test: PASS core only_bare only in bare
pictura self-test: SUMMARY passed=2 failed=0 skipped=0
"""

SELFTEST_PSD = """pictura self-test: FAIL core shared 7 boom
pictura self-test: PASS core only_psd only in psd
pictura self-test: SUMMARY passed=1 failed=1 skipped=0
"""


def run_self_check():
    suites, failures = parse_junit(PASS_JUNIT)
    assert suites["pictura-core"].passed == 2, suites
    assert suites["pictura-codec"].passed == 1, suites
    assert not failures, failures

    suites, failures = parse_junit(FAIL_JUNIT)
    assert suites["pictura-codec"].failed == 1, suites
    assert suites["pictura-codec"].skipped == 1, suites
    assert suites["pictura-codec"].passed == 0, suites
    assert len(failures) == 1, failures
    assert failures[0][0] == "pictura-codec::oracle::bad", failures
    assert "assertion" in failures[0][1], failures

    counts, failures, seen = parse_doctests(DOCTEST_OK)
    assert seen and counts.passed == 3 and counts.failed == 0, (counts, failures)

    counts, failures, seen = parse_doctests(DOCTEST_FAIL)
    assert seen and counts.passed == 2 and counts.failed == 1, counts
    assert len(failures) == 1 and "foo::bar" in failures[0][0], failures

    _, _, seen = parse_doctests("no summary here")
    assert not seen

    suites, totals, failures = parse_selftest(SELFTEST)
    assert suites["core"].passed == 1, suites
    assert suites["core"].skipped == 1, suites
    assert suites["m47"].failed == 1, suites
    assert (totals.passed, totals.failed, totals.skipped) == (1, 1, 1), totals
    assert len(failures) == 1, failures
    assert failures[0][0] == "m47::compact_shade (exit 195)", failures
    assert failures[0][1] == "M47 compact shade", failures

    suites, totals, failures = merge_selftest_streams([SELFTEST_BARE, SELFTEST_PSD])
    assert (totals.passed, totals.failed, totals.skipped) == (2, 1, 0), totals
    assert suites["core"].total() == 3, suites
    assert len(failures) == 1, failures
    assert failures[0][0] == "core::shared (exit 7)", failures
    assert failures[0][1] == "boom", failures

    suites, totals, _ = merge_selftest_streams([SELFTEST_BARE])
    assert (totals.passed, totals.failed, totals.skipped) == (2, 0, 0), totals

    _, _, human = parse_selftest(
        "pictura self-test: document_size=4000\npictura self-test: FAIL: nope"
    )
    assert not human, human

    assert exit_code([], []) == 0
    assert exit_code([("x", "")], []) == 1
    assert exit_code([], ["e"]) == 1

    layer = [Layer("Rust suites (nextest)", {"pictura-core": Counts(2)})]
    assert "\x1b[" not in render(layer, [], False), "color never leaked ANSI"
    assert "\x1b[" in render(layer, [], True), "color always missing ANSI"

    print("report_tests.py: self-check OK")
    return 0


def main(argv=None):
    args = _parse_args(argv)
    if args.self_check:
        return run_self_check()

    errors = []
    layers, failures = build_layers(args, errors)
    color = _color_enabled("never" if args.no_color else args.color)
    sys.stdout.write(render(layers, failures, color) + "\n")
    for error in errors:
        print(f"report_tests.py: error: {error}", file=sys.stderr)
    return exit_code(failures, errors)


if __name__ == "__main__":
    sys.exit(main())
