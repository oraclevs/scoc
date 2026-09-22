# SCOC

**SCOC** is the SPA Command Output Converter: a Rust-native parser library that turns ordinary human-readable command/file output into structured `serde_json::Value` data for Spar/Sparsh.

## Parser platform scope

This source snapshot registers **274 canonical parsers**:

- **217 JC-derived Linux/macOS/generic targets** from JC **1.26.0**, pinned to commit `73fa7d5572dd730076723bd6280786bb9101d32f`.
- **57 SCOC-native developer/DevOps targets** across Docker, Kubernetes, Helm, Git, GitHub CLI, Rust, Python, JavaScript, Java/JVM, Go, Flutter, Dart, and Terraform.
- **6 Windows-only JC parsers are cataloged but deliberately deferred**: `dir`, `ipconfig`, `net-localgroup`, `net-user`, `route-print`, and `systeminfo`.
- **20 JC streaming names** such as `ping-s`, `csv-s`, and `git-log-s` resolve as aliases/capabilities of their canonical parser and do not inflate the canonical count.

The authoritative generated list is `docs/SCOC_PARSER_CATALOG.md`. The source manifests are `compatibility/jc-inventory.toml`, `compatibility/parser-matrix.toml`, and `compatibility/native-catalog.toml`.

## Examples

SCOC is a library. Spar/Sparsh can expose registry entries through `from <parser>` / `from scoc::<parser>`:

```text
uname -a | from uname
docker ps | from docker-ps
docker stats --no-stream | from docker-stats
kubectl get pods -A -o wide | from kubectl-get
cargo tree | from cargo-tree
pip freeze | from pip-freeze
go env | from go-env
flutter doctor | from flutter-doctor
terraform workspace list | from terraform-workspace-list
```

When a tool already emits a stable machine-readable format, callers can keep using the direct escape hatch instead of re-parsing pretty output, for example `from json` or `from yaml`.

## Verification states matter

`implemented` means a parser has a registered implementation and metadata. It does **not** mean differential compatibility was proven. Only actual Rust test/Clippy execution and, for JC-derived parsers, an actual run against the pinned JC 1.26.0 oracle can promote verification state.

This archive was assembled in an environment without `cargo`/`rustc`, so its Rust compile/test/Clippy gate and full JC differential gate remain unverified here. Run the authoritative local gate on a Rust-enabled machine:

```bash
./verify-scoc.sh
```

If JC 1.26.0 is installed in the same environment, include the differential corpus:

```bash
./verify-scoc.sh --with-jc-diff
```

The differential runner is manifest-driven. You can also check one canonical parser with a checked-in fixture using:

```bash
python3 compatibility/diff.py --parser df
```

See `SCOC_HANDOFF.md` for measured counts and the exact unverified/verified status of this snapshot.
