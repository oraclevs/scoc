#!/usr/bin/env python3
"""Synchronize SCOC's parser inventory from a pinned JC 1.26.0 source tree.

The source is parsed with ast; it is never imported or executed.
"""
from __future__ import annotations
import argparse, ast, json
from pathlib import Path

BASELINE_VERSION = "1.26.0"
BASELINE_COMMIT = "73fa7d5572dd730076723bd6280786bb9101d32f"


def literal_assignments(path: Path) -> dict[str, object]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    result: dict[str, object] = {}
    for node in ast.walk(tree):
        if isinstance(node, (ast.Assign, ast.AnnAssign)):
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            value_node = node.value
            for target in targets:
                if isinstance(target, ast.Name):
                    try:
                        result[target.id] = ast.literal_eval(value_node)
                    except Exception:
                        pass
                elif isinstance(target, ast.Attribute) and isinstance(target.value, ast.Name) and target.value.id == "info":
                    try:
                        result[target.attr] = ast.literal_eval(value_node)
                    except Exception:
                        pass
    return result


def parser_names(lib_path: Path) -> list[str]:
    assignments = literal_assignments(lib_path)
    parsers = assignments.get("parsers")
    if not isinstance(parsers, list) or not all(isinstance(x, str) for x in parsers):
        raise ValueError(f"could not read literal parsers list from {lib_path}")
    return parsers


def release_target(compatible: list[str]) -> str:
    normalized = set(compatible)
    if normalized == {"win32"}:
        return "windows-deferred"
    if normalized & {"linux", "darwin"}:
        return "linux-macos-generic"
    return "generic"


def build_inventory(source: Path) -> dict[str, object]:
    lib = source / "jc" / "lib.py"
    parsers_dir = source / "jc" / "parsers"
    names = parser_names(lib)
    name_set = set(names)
    streaming = {name[:-2]: name for name in names if name.endswith("-s") and name[:-2] in name_set}
    canonical = [name for name in names if not (name.endswith("-s") and name[:-2] in name_set)]
    entries: dict[str, dict[str, object]] = {}
    for name in canonical:
        module = parsers_dir / f"{name.replace('-', '_')}.py"
        meta = literal_assignments(module) if module.exists() else {}
        compatible = meta.get("compatible", [])
        if not isinstance(compatible, list): compatible = []
        entry = {
            "name": name,
            "module": module.name,
            "version": str(meta.get("version", "unknown-pinned")),
            "compatible": compatible,
            "target": release_target(compatible),
            "hidden": bool(meta.get("hidden", False)),
            "deprecated": bool(meta.get("deprecated", False)),
        }
        if name in streaming:
            stream_name = streaming[name]
            stream_module = parsers_dir / f"{stream_name.replace('-', '_')}.py"
            stream_meta = literal_assignments(stream_module) if stream_module.exists() else {}
            entry.update({
                "streaming_name": stream_name,
                "streaming_module": stream_module.name,
                "streaming_version": str(stream_meta.get("version", "unknown-pinned")),
            })
        entries[name] = entry
    return {
        "baseline": {"version": BASELINE_VERSION, "commit": BASELINE_COMMIT},
        "parser_names_in_lib": len(names),
        "canonical_after_stream_collapse": len(entries),
        "parsers": entries,
    }


def toml_quote(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def to_toml(inventory: dict[str, object]) -> str:
    baseline = inventory["baseline"]
    out = ["[baseline]", f'version = {toml_quote(baseline["version"])}', f'commit = {toml_quote(baseline["commit"])}', ""]
    for name, meta in inventory["parsers"].items():
        out += [f"[parsers.{name}]", f'name = {toml_quote(name)}', f'module = {toml_quote(meta["module"])}', f'version = {toml_quote(meta["version"])}']
        out.append("compatible = [" + ", ".join(toml_quote(x) for x in meta["compatible"]) + "]")
        out += [f'target = {toml_quote(meta["target"])}', f'hidden = {str(meta["hidden"]).lower()}', f'deprecated = {str(meta["deprecated"]).lower()}']
        for key in ("streaming_name", "streaming_module", "streaming_version"):
            if key in meta: out.append(f'{key} = {toml_quote(meta[key])}')
        out.append("")
    return "\n".join(out) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", type=Path, help="local JC source tree pinned to the baseline commit")
    ap.add_argument("--output", type=Path, default=Path(__file__).with_name("jc-inventory.toml"))
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    if not args.source:
        if args.check:
            if not args.output.exists(): raise SystemExit("inventory does not exist")
            print(f"inventory present: {args.output}")
            return 0
        ap.error("--source is required for regeneration")
    inventory = build_inventory(args.source)
    text = to_toml(inventory)
    if args.check:
        if args.output.read_text(encoding="utf-8") != text:
            print("inventory drift", file=__import__('sys').stderr); return 1
    else:
        args.output.write_text(text, encoding="utf-8")
    print(f"jc_parser_names_in_lib = {inventory['parser_names_in_lib']}")
    print(f"canonical_after_stream_collapse = {inventory['canonical_after_stream_collapse']}")
    print(f"linux_macos_generic_targets = {sum(1 for x in inventory['parsers'].values() if x['target'] != 'windows-deferred')}")
    print(f"windows_deferred = {sum(1 for x in inventory['parsers'].values() if x['target'] == 'windows-deferred')}")
    print(f"streaming_aliases = {sum(1 for x in inventory['parsers'].values() if 'streaming_name' in x)}")
    return 0

if __name__ == "__main__": raise SystemExit(main())
