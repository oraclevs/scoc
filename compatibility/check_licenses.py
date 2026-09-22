#!/usr/bin/env python3
"""Offline licensing/provenance closure check for SCOC parser assets."""
from __future__ import annotations

import sys
import tomllib
from pathlib import Path

PINNED_JC = "73fa7d5572dd730076723bd6280786bb9101d32f"


def load_toml(path: Path):
    with path.open("rb") as fh:
        return tomllib.load(fh)


def collect_notice_ids(value, out: set[str]):
    if isinstance(value, dict):
        for key, child in value.items():
            if key in {"license_notice", "license_notices", "third_party_notice", "third_party_notices"}:
                if isinstance(child, str):
                    out.add(child)
                elif isinstance(child, list):
                    out.update(str(item) for item in child)
            collect_notice_ids(child, out)
    elif isinstance(value, list):
        for child in value:
            collect_notice_ids(child, out)


def validate(root: Path) -> list[str]:
    errors: list[str] = []
    notices = root / "THIRD_PARTY_LICENSES"
    jc_license = notices / "JC-MIT.txt"
    if not jc_license.is_file():
        errors.append("missing THIRD_PARTY_LICENSES/JC-MIT.txt")

    inventory_path = root / "compatibility" / "jc-inventory.toml"
    matrix_path = root / "compatibility" / "parser-matrix.toml"
    if not inventory_path.is_file() or not matrix_path.is_file():
        return errors + ["missing JC inventory or parser matrix"]
    inventory = load_toml(inventory_path)
    matrix = load_toml(matrix_path)
    baseline = inventory.get("baseline", {})
    if baseline.get("version") != "1.26.0":
        errors.append(f"JC baseline version is {baseline.get('version')!r}, expected '1.26.0'")
    if baseline.get("commit") != PINNED_JC:
        errors.append(f"JC baseline commit is {baseline.get('commit')!r}, expected {PINNED_JC}")

    jc_rows = inventory.get("parsers", {})
    matrix_rows = matrix.get("parsers", {})
    if set(jc_rows) != set(matrix_rows):
        missing = sorted(set(jc_rows) - set(matrix_rows))
        extra = sorted(set(matrix_rows) - set(jc_rows))
        if missing:
            errors.append(f"parser matrix missing JC rows: {missing}")
        if extra:
            errors.append(f"parser matrix has non-inventory JC rows: {extra}")
    for name, row in matrix_rows.items():
        if row.get("origin") != "jc":
            errors.append(f"matrix row {name} does not declare origin='jc'")

    jc_root = root / "tests" / "fixtures" / "jc-1.26.0"
    for fixture in sorted(jc_root.rglob("*.out")):
        meta = fixture.with_suffix(".meta.toml")
        if not meta.is_file():
            errors.append(f"JC fixture lacks adjacent provenance: {fixture.relative_to(root)}")
            continue
        data = load_toml(meta)
        if data.get("upstream") != "kellyjonbrazil/jc":
            errors.append(f"JC fixture has unexpected upstream: {meta.relative_to(root)}")
        if data.get("commit") != PINNED_JC:
            errors.append(f"JC fixture is not pinned to baseline commit: {meta.relative_to(root)}")
        parser = data.get("parser")
        if parser not in jc_rows:
            errors.append(f"JC fixture references unknown parser {parser!r}: {meta.relative_to(root)}")
        if not str(data.get("provenance", "")).strip():
            errors.append(f"JC fixture lacks provenance text: {meta.relative_to(root)}")

    aggregate = jc_root / "PROVENANCE.toml"
    if not aggregate.is_file():
        errors.append("missing tests/fixtures/jc-1.26.0/PROVENANCE.toml")
    else:
        rows = load_toml(aggregate).get("fixtures", [])
        paths = {row.get("path") for row in rows}
        expected = {p.relative_to(root).as_posix() for p in jc_root.rglob("*.out")}
        if paths != expected:
            errors.append("JC aggregate provenance does not exactly cover .out fixtures")

    native_root = root / "tests" / "fixtures" / "native"
    native_catalog = load_toml(root / "compatibility" / "native-catalog.toml").get("parsers", {})
    for fixture in sorted(native_root.rglob("*.out")):
        meta = fixture.with_suffix(".meta.toml")
        if not meta.is_file():
            errors.append(f"native fixture lacks adjacent provenance: {fixture.relative_to(root)}")
            continue
        data = load_toml(meta)
        parser = data.get("parser")
        if parser not in native_catalog:
            errors.append(f"native fixture references unknown parser {parser!r}: {meta.relative_to(root)}")
        for field in ("tool_version", "source_kind", "source", "verification"):
            if not str(data.get(field, "")).strip():
                errors.append(f"native fixture missing {field}: {meta.relative_to(root)}")

    native_aggregate = native_root / "PROVENANCE.toml"
    if not native_aggregate.is_file():
        errors.append("missing tests/fixtures/native/PROVENANCE.toml")
    else:
        rows = load_toml(native_aggregate).get("fixtures", [])
        paths = {row.get("path") for row in rows}
        expected = {p.relative_to(root).as_posix() for p in native_root.rglob("*.out")}
        if paths != expected:
            errors.append("native aggregate provenance does not exactly cover .out fixtures")

    declared: set[str] = set()
    for toml_path in (root / "compatibility").glob("*.toml"):
        collect_notice_ids(load_toml(toml_path), declared)
    existing = {p.name for p in notices.iterdir() if p.is_file()} if notices.is_dir() else set()
    for notice in sorted(declared):
        if notice not in existing:
            errors.append(f"declared third-party notice does not exist: {notice}")
    return errors


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    errors = validate(root)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    jc_count = len(list((root / "tests/fixtures/jc-1.26.0").rglob("*.out")))
    native_count = len(list((root / "tests/fixtures/native").rglob("*.out")))
    print(f"license/provenance OK: {jc_count} JC fixtures, {native_count} native contract fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
