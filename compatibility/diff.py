#!/usr/bin/env python3
"""Differential runner against the pinned JC 1.26.0 oracle.

Fixture discovery is manifest-driven so parser names containing dashes and streaming
aliases are never inferred from directory layout.
"""
import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORACLE = ROOT / "compatibility" / "jc_oracle.py"
CASES = ROOT / "tests" / "fixtures" / "jc-1.26.0" / "cases.json"


def run_json(cmd, *, stdin=None):
    proc = subprocess.run(cmd, input=stdin, text=True, capture_output=True, cwd=ROOT)
    if proc.returncode:
        if proc.stdout:
            sys.stderr.write(proc.stdout)
        if proc.stderr:
            sys.stderr.write(proc.stderr)
        raise SystemExit(proc.returncode)
    return json.loads(proc.stdout)


def load_cases():
    data = json.loads(CASES.read_text(encoding="utf-8"))
    if not isinstance(data, list):
        raise SystemExit(f"invalid fixture manifest: {CASES}")
    return data


def compare(case):
    parser = case["parser"]
    fixture = ROOT / case["input"]
    raw = bool(case.get("raw", False))
    streaming = bool(case.get("streaming", False))
    ignore_errors = bool(case.get("ignore_errors", False))
    request = json.dumps({
        "parser": parser,
        "streaming_parser": case.get("streaming_parser"),
        "input_path": str(fixture),
        "raw": raw,
        "streaming": streaming,
        "ignore_errors": ignore_errors,
    })
    expected = run_json([sys.executable, str(ORACLE)], stdin=request)
    cmd = [
        "cargo", "run", "--quiet", "--example", "compat_driver", "--",
        "--parser", parser, "--fixture", str(fixture),
    ]
    if raw:
        cmd.append("--raw")
    if streaming:
        cmd.append("--streaming")
    if ignore_errors:
        cmd.append("--ignore-errors")
    actual = run_json(cmd)
    if expected != actual:
        print(json.dumps({"parser": parser, "fixture": str(fixture), "expected": expected, "actual": actual}, indent=2), file=sys.stderr)
        return False
    mode = "stream" if streaming else "batch"
    print(f"PASS {parser} [{mode}{'/raw' if raw else ''}]: {fixture.relative_to(ROOT)}")
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parser", help="canonical parser name; runs all manifest cases for that parser")
    ap.add_argument("--fixture", type=Path, help="one explicit fixture (requires --parser)")
    ap.add_argument("--all-fixtures", action="store_true")
    ap.add_argument("--raw", action="store_true", help="override explicit fixture case to raw mode")
    ap.add_argument("--streaming", action="store_true", help="override explicit fixture case to streaming mode")
    ap.add_argument("--ignore-errors", action="store_true", help="override explicit fixture case ignore-errors mode")
    args = ap.parse_args()

    if shutil.which("cargo") is None:
        print("cargo is required for SCOC differential execution", file=sys.stderr)
        return 2

    try:
        import jc  # noqa: F401
        import jc.lib
        if getattr(jc.lib, "__version__", None) != "1.26.0":
            print(f"jc 1.26.0 is required; found {getattr(jc.lib, '__version__', None)!r}", file=sys.stderr)
            return 2
    except ImportError:
        print("jc 1.26.0 is required for differential execution", file=sys.stderr)
        return 2

    if args.fixture:
        if not args.parser:
            ap.error("--fixture requires --parser")
        path = args.fixture if args.fixture.is_absolute() else (ROOT / args.fixture)
        case = {
            "parser": args.parser,
            "input": str(path.relative_to(ROOT)),
            "raw": args.raw,
            "streaming": args.streaming,
            "ignore_errors": args.ignore_errors,
        }
        cases = [case]
    else:
        if not args.all_fixtures and not args.parser:
            ap.error("provide --parser, --fixture, or --all-fixtures")
        cases = load_cases()
        if args.parser:
            cases = [case for case in cases if case["parser"] == args.parser]

    if not cases:
        label = args.parser or "the vendored parser set"
        print(f"no differential fixtures found for {label}", file=sys.stderr)
        return 2

    ok = True
    for case in cases:
        ok = compare(case) and ok
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
