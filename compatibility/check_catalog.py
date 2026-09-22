#!/usr/bin/env python3
from __future__ import annotations
import sys, tomllib
from pathlib import Path
from generate_catalog import outputs, ROOT

def main():
    jc=tomllib.loads((ROOT/'compatibility/jc-inventory.toml').read_text())['parsers']
    matrix=tomllib.loads((ROOT/'compatibility/parser-matrix.toml').read_text())['parsers']
    native=tomllib.loads((ROOT/'compatibility/native-catalog.toml').read_text())['parsers']
    errors=[]
    if set(jc)!=set(matrix): errors.append('JC inventory/matrix key mismatch')
    if len(jc)!=223: errors.append(f'expected 223 JC canonical, got {len(jc)}')
    deferred={n for n,m in jc.items() if m['target']=='windows-deferred'}
    if deferred!={'dir','ipconfig','net-localgroup','net-user','route-print','systeminfo'}: errors.append(f'unexpected deferred set: {sorted(deferred)}')
    if len(native)!=57: errors.append(f'expected 57 native canonical, got {len(native)}')
    expected_ecos={'docker','kubernetes','helm','git','github','rust','python','javascript','java','go','flutter','dart','terraform'}
    actual_ecos={m['ecosystem'] for m in native.values()}
    if actual_ecos!=expected_ecos: errors.append(f'native ecosystems mismatch: {sorted(actual_ecos)}')
    # Source file closure.
    jc_files={p.stem.replace('_','-') for p in (ROOT/'src/parsers/jc').glob('*.rs') if p.name not in {'mod.rs','common.rs'}}
    native_files={p.stem.replace('_','-') for p in (ROOT/'src/parsers/native').glob('*/*.rs') if p.name!='mod.rs'}
    registered={n for n,m in jc.items() if m['target']!='windows-deferred'}
    if registered-jc_files: errors.append(f'missing JC modules: {sorted(registered-jc_files)}')
    if set(native)-native_files: errors.append(f'missing native modules: {sorted(set(native)-native_files)}')
    for path,text in outputs().items():
        if not path.exists() or path.read_text()!=text: errors.append(f'generated drift: {path.relative_to(ROOT)}')
    if errors:
        for e in errors: print(e,file=sys.stderr)
        return 1
    print(f'catalog OK: 223 JC canonical / 6 deferred / 217 registered JC / 57 native / 274 total registered')
    return 0
if __name__=='__main__': raise SystemExit(main())
