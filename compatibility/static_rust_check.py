#!/usr/bin/env python3
"""Cheap source-shape checks for environments without rustc.

This is not a replacement for `cargo check`; it catches generator drift and a few
classes of syntax/registry mistakes before the Rust gate is available.
"""
from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load(path):
    with path.open('rb') as f: return tomllib.load(f)


def strip_strings_comments(text: str) -> str:
    """Mask comments/string/char/raw-string contents while preserving delimiters."""
    out=[]; i=0; n=len(text); block_depth=0
    while i<n:
        ch=text[i]; nxt=text[i+1] if i+1<n else ''
        # line comments
        if ch=='/' and nxt=='/':
            while i<n and text[i]!='\n': out.append(' '); i+=1
            continue
        # nested block comments
        if ch=='/' and nxt=='*':
            block_depth=1; out.extend('  '); i+=2
            while i<n and block_depth:
                if i+1<n and text[i:i+2]=='/*': block_depth+=1; out.extend('  '); i+=2; continue
                if i+1<n and text[i:i+2]=='*/': block_depth-=1; out.extend('  '); i+=2; continue
                out.append('\n' if text[i]=='\n' else ' '); i+=1
            continue
        # raw strings: r"...", r#"..."#, br#"..."#
        raw_start=None
        for prefix in ('br','r'):
            if text.startswith(prefix, i):
                j=i+len(prefix); hashes=0
                while j<n and text[j]=='#': hashes+=1; j+=1
                if j<n and text[j]=='"': raw_start=(j,hashes); break
        if raw_start:
            quote_pos,hashes=raw_start
            endtok='"' + ('#'*hashes)
            while i<=quote_pos: out.append(' '); i+=1
            pos=text.find(endtok,i)
            if pos<0:
                out.extend('\n' if c=='\n' else ' ' for c in text[i:]); i=n; continue
            out.extend('\n' if c=='\n' else ' ' for c in text[i:pos+len(endtok)])
            i=pos+len(endtok); continue
        # normal string
        if ch=='"':
            out.append(' '); i+=1
            while i<n:
                if text[i]=='\\': out.append(' '); i+=1; out.append(' ' if i<n else ''); i+=1; continue
                c=text[i]; out.append('\n' if c=='\n' else ' '); i+=1
                if c=='"': break
            continue
        # char literal versus lifetime. A lifetime does not close with a quote.
        if ch=="'":
            is_char = False
            if i+2<n and text[i+2]=="'": is_char=True
            if i+3<n and text[i+1]=='\\' and text[i+3]=="'": is_char=True
            if is_char:
                out.append(' '); i+=1
                while i<n:
                    if text[i]=='\\': out.append(' '); i+=1; out.append(' ' if i<n else ''); i+=1; continue
                    c=text[i]; out.append(' '); i+=1
                    if c=="'": break
                continue
            out.append(ch); i+=1; continue
        out.append(ch); i+=1
    return ''.join(out)


def balance(path: Path, text: str, errors: list[str]):
    clean=strip_strings_comments(text)
    pairs={')':'(',']':'[','}':'{'}; stack=[]
    for pos,ch in enumerate(clean):
        if ch in '([{': stack.append((ch,pos))
        elif ch in pairs:
            if not stack or stack[-1][0]!=pairs[ch]:
                errors.append(f'{path.relative_to(ROOT)}: unbalanced {ch} at byte {pos}'); return
            stack.pop()
    if stack: errors.append(f'{path.relative_to(ROOT)}: unclosed {stack[-1][0]}')

def aliases_from(text: str) -> list[str]:
    consts={}
    for name, body in re.findall(r'(?:const|static)\s+(\w+)\s*:[^=]+\s*=\s*\[(.*?)\]\s*;', text, re.S):
        consts[name]=re.findall(r'"([^"]+)"', body)
    m=re.search(r'aliases\s*:\s*&\[(.*?)\]', text, re.S)
    if m: return re.findall(r'"([^"]+)"', m.group(1))
    m=re.search(r'aliases\s*:\s*&([A-Z][A-Z0-9_]*)', text)
    return consts.get(m.group(1),[]) if m else []


def main() -> int:
    errors=[]
    rust_files=sorted((ROOT/'src').rglob('*.rs'))+sorted((ROOT/'tests').rglob('*.rs'))+sorted((ROOT/'examples').rglob('*.rs'))
    for path in rust_files:
        text=path.read_text()
        balance(path, text, errors)
        if 'pub static PARSER: GenericParser' in text:
            errors.append(f'{path.relative_to(ROOT)}: public static exposes crate-private GenericParser')
        if 'compatible-target' in text:
            errors.append(f'{path.relative_to(ROOT)}: unverified compatibility wording')

    inv=load(ROOT/'compatibility/jc-inventory.toml')['parsers']
    native=load(ROOT/'compatibility/native-catalog.toml')['parsers']
    expected={n for n,r in inv.items() if r['target']!='windows-deferred'}|set(native)
    source={}
    for path in sorted((ROOT/'src/parsers/jc').glob('*.rs'))+sorted((ROOT/'src/parsers/native').glob('*/*.rs')):
        if path.name in {'mod.rs','common.rs'}: continue
        text=path.read_text()
        m=re.search(r'name\s*:\s*"([^"]+)"',text)
        if not m:
            errors.append(f'{path.relative_to(ROOT)}: no descriptor name found'); continue
        name=m.group(1)
        if name in source: errors.append(f'duplicate descriptor name {name}: {source[name]} and {path.relative_to(ROOT)}')
        source[name]=path.relative_to(ROOT)
    if set(source)!=expected:
        if expected-set(source): errors.append(f'missing descriptor modules: {sorted(expected-set(source))}')
        if set(source)-expected: errors.append(f'unexpected descriptor modules: {sorted(set(source)-expected)}')

    # Source-level alias collision check.
    keys={name:name for name in source}
    alias_count=0
    for name,path in source.items():
        text=(ROOT/path).read_text()
        for alias in aliases_from(text):
            key=alias.strip().lower()
            alias_count+=1
            if key in keys and keys[key]!=name:
                errors.append(f'alias collision: {alias!r} on {name} collides with {keys[key]}')
            else: keys[key]=name

    if errors:
        for e in errors: print(e,file=sys.stderr)
        return 1
    print(f'static Rust generator/registry shape OK: {len(rust_files)} .rs files, {len(source)} canonical descriptors, {alias_count} declared aliases')
    print('NOTE: this does not replace cargo fmt/clippy/test')
    return 0

if __name__=='__main__': raise SystemExit(main())
