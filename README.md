# SCOC

SCOC is the Spar Command Output Converter: a Rust-native parser library that turns ordinary human-readable command and file output into structured `serde_json::Value` data for Spar/Sparsh.

## Parser platform scope

SCOC registers 274 canonical parsers:

- 217 JC-derived Linux/macOS/generic targets from JC 1.26.0, pinned to commit `73fa7d5572dd730076723bd6280786bb9101d32f`.
- 57 SCOC-native developer/DevOps targets across Docker, Kubernetes, Helm, Git, GitHub CLI, Rust, Python, JavaScript, Java/JVM, Go, Flutter, Dart, and Terraform.
- 6 Windows-only JC parsers are cataloged but deliberately deferred: `dir`, `ipconfig`, `net-localgroup`, `net-user`, `route-print`, and `systeminfo`.
- 20 JC streaming names such as `ping-s`, `csv-s`, and `git-log-s` resolve as aliases of their canonical parser and don't add to the canonical count.

The generated list is `docs/SCOC_PARSER_CATALOG.md`. The source manifests are `compatibility/jc-inventory.toml`, `compatibility/parser-matrix.toml`, and `compatibility/native-catalog.toml`.

## Examples

SCOC is a library. Spar/Sparsh expose registry entries through `from <parser>` / `from scoc::<parser>`:

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

If a tool already emits a stable machine-readable format, there's no need to parse pretty output for it — use `from json` or `from yaml` directly.

## Verification states matter

`implemented` means a parser has a registered implementation and metadata. It doesn't mean differential compatibility against JC has been proven for that parser: 61 of the JC-derived parsers have been differentially verified against the pinned JC 1.26.0 oracle so far, and the other 156 are structured fallback implementations pending that pass.

The crate itself has been verified on a real Rust toolchain: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features` all pass, along with the Python-side checks (catalog, license, and inventory consistency). Run the same local gate yourself:

```bash
./verify-scoc.sh
```

If JC 1.26.0 is installed in the same environment, include the differential corpus:

```bash
./verify-scoc.sh --with-jc-diff
```

The differential runner is manifest-driven. You can also check one canonical parser against its checked-in fixture:

```bash
python3 compatibility/diff.py --parser df
```

See `SCOC_HANDOFF.md` for the parser-by-parser verification status.
