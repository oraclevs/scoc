#!/usr/bin/env python3
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORACLE = ROOT / "compatibility" / "jc_oracle.py"


def run_json(cmd, *, stdin=None):
    proc = subprocess.run(cmd, input=stdin, text=True, capture_output=True, cwd=ROOT)
    if proc.returncode:
        sys.stderr.write(proc.stderr)
        raise SystemExit(proc.returncode)
    return json.loads(proc.stdout)


def compare(parser, fixture, raw, streaming, ignore_errors):
    request = json.dumps({
        "parser": parser,
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
        print(json.dumps({"fixture": str(fixture), "expected": expected, "actual": actual}, indent=2), file=sys.stderr)
        return False
    print(f"PASS {parser}: {fixture}")
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parser", help="parser name; omit with --all-fixtures to infer from fixture directories")
    ap.add_argument("--fixture", type=Path)
    ap.add_argument("--all-fixtures", action="store_true")
    ap.add_argument("--raw", action="store_true")
    ap.add_argument("--streaming", action="store_true")
    ap.add_argument("--ignore-errors", action="store_true")
    args = ap.parse_args()

    if args.fixture:
        if not args.parser:
            ap.error("--fixture requires --parser")
        cases = [(args.parser, args.fixture)]
    elif args.all_fixtures:
        fixtures = sorted((ROOT / "tests" / "fixtures" / "jc-1.26.0").rglob("*.out"))
        if args.parser:
            fixtures = [
                path for path in fixtures
                if path.parent.name == args.parser or args.parser in path.name
            ]
            cases = [(args.parser, path) for path in fixtures]
        else:
            cases = [(path.parent.name, path) for path in fixtures]
    else:
        ap.error("provide --fixture or --all-fixtures")

    if not cases:
        label = args.parser or "the vendored parser set"
        print(f"no fixtures found for {label}", file=sys.stderr)
        return 2
    ok = all(
        compare(parser, fixture, args.raw, args.streaming, args.ignore_errors)
        for parser, fixture in cases
    )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
