#!/usr/bin/env python3
from __future__ import annotations
import argparse, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def load():
    jc=tomllib.loads((ROOT/'compatibility/jc-inventory.toml').read_text())['parsers']
    matrix=tomllib.loads((ROOT/'compatibility/parser-matrix.toml').read_text())['parsers']
    native=tomllib.loads((ROOT/'compatibility/native-catalog.toml').read_text())['parsers']
    return jc,matrix,native

def rust_text(native):
    out=["pub(crate) fn native_ecosystem(name: &str) -> Option<&'static str> {","    match name {"]
    for name,meta in native.items(): out.append(f'        "{name}" => Some("{meta["ecosystem"]}"),')
    out += ["        _ => None,","    }","}","", "pub(crate) fn variants(name: &str) -> &'static [&'static str] {", "    match name {"]
    for name,meta in native.items():
        vals=', '.join('"'+v.replace('\\','\\\\').replace('"','\\"')+'"' for v in meta.get('variants',['default']))
        out.append(f'        "{name}" => &[{vals}],')
    out += ['        _ => &["default"],','    }','}']
    return '\n'.join(out)+'\n'

def catalog_doc(jc,matrix,native):
    rows=['# SCOC Parser Catalog','', 'Generated from the pinned JC inventory and SCOC native catalog. Streaming aliases are capabilities and do not increase canonical counts.','', '| Parser | Origin | Ecosystem/Target | Status | Streaming |','|---|---|---|---|---|']
    for name,meta in jc.items():
        m=matrix[name]
        rows.append(f'| `{name}` | JC 1.26.0 | {meta["target"]} | {m["status"]} | {meta.get("streaming_name", "—")} |')
    for name,meta in native.items():
        rows.append(f'| `{name}` | SCOC native | {meta["ecosystem"]} | {meta["status"]} | — |')
    rows += ['', f'**JC canonical:** {len(jc)}  ', f'**Windows deferred:** {sum(1 for x in jc.values() if x["target"]=="windows-deferred")}  ', f'**Native canonical:** {len(native)}  ', f'**Registered canonical:** {sum(1 for x in jc.values() if x["target"]!="windows-deferred")+len(native)}']
    return '\n'.join(rows)+'\n'

def smoke_doc(jc,native):
    known={
      'docker-ps':'docker ps | from docker-ps','docker-images':'docker images | from docker-images','docker-stats':'docker stats --no-stream | from docker-stats',
      'kubectl-get':'kubectl get pods -A -o wide | from kubectl-get','kubectl-top':'kubectl top pods -A | from kubectl-top','helm-list':'helm list -A | from helm-list',
      'git-status':'git status --short | from git-status','git-branch':'git branch -vv | from git-branch','gh-pr-list':'gh pr list | from gh-pr-list',
      'cargo-tree':'cargo tree | from cargo-tree','rustup-toolchains':'rustup toolchain list | from rustup-toolchains','rustc-version':'rustc --version --verbose | from rustc-version',
      'pip-freeze':'pip freeze | from pip-freeze','pip-check':'pip check | from pip-check','npm-run':'npm run | from npm-run','npm-ls':'npm ls | from npm-ls',
      'java-version':'java -version 2>&1 | from java-version','go-version':'go version | from go-version','go-env':'go env | from go-env',
      'flutter-doctor':'flutter doctor -v | from flutter-doctor','flutter-devices':'flutter devices | from flutter-devices','terraform-state-list':'terraform state list | from terraform-state-list','terraform-workspace-list':'terraform workspace list | from terraform-workspace-list',
    }
    out=['# SCOC Manual Smoke Tests','', 'Generated checklist. Run only commands available on the current host.','']
    for name in ['uname','df','ps','ls','ping','git-log']:
        if name in jc: out.append(f'- [ ] `{name}` — verify a representative local `{name}` output through `from {name}`')
    for name in native:
        if name in known: out.append(f'- [ ] `{known[name]}`')
        else: out.append(f'- [ ] `{name}` — capture the documented default human-readable command output and pipe it to `from {name}`')
    return '\n'.join(out)+'\n'

def outputs():
    jc,matrix,native=load()
    return {
      ROOT/'src/catalog_generated.rs':rust_text(native),
      ROOT/'docs/SCOC_PARSER_CATALOG.md':catalog_doc(jc,matrix,native),
      ROOT/'docs/SCOC_MANUAL_SMOKE_TESTS.md':smoke_doc(jc,native),
    }

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--check',action='store_true'); args=ap.parse_args()
    ok=True
    for path,text in outputs().items():
        if args.check:
            if not path.exists() or path.read_text()!=text:
                print(f'drift: {path.relative_to(ROOT)}'); ok=False
        else:
            path.parent.mkdir(parents=True,exist_ok=True); path.write_text(text)
    return 0 if ok else 1
if __name__=='__main__': raise SystemExit(main())
