#[test]
fn baseline_files_match_compiled_baseline() {
    let baseline = scoc::compatibility_baseline();
    let toml = include_str!("../compatibility/jc-baseline.toml");
    let matrix = include_str!("../compatibility/parser-matrix.toml");
    assert!(toml.contains(&format!("version = \"{}\"", baseline.version)));
    assert!(toml.contains(&format!("commit = \"{}\"", baseline.commit)));
    assert!(matrix.contains(&format!("version = \"{}\"", baseline.version)));
    assert!(matrix.contains(&format!("commit = \"{}\"", baseline.commit)));
    assert!(
        !matrix.contains("status = \"compatible\""),
        "unverified parsers must not be promoted"
    );
}

#[test]
fn every_vendored_fixture_has_provenance_metadata() {
    let root = std::path::Path::new("tests/fixtures/jc-1.26.0");
    let mut stack = vec![root.to_path_buf()];
    let mut fixture_count = 0usize;

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", dir.display()))
        {
            let entry = entry.unwrap_or_else(|error| panic!("fixture directory entry: {error}"));
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("out") {
                continue;
            }
            fixture_count += 1;
            let metadata_path = path.with_extension("meta.toml");
            let metadata = std::fs::read_to_string(&metadata_path).unwrap_or_else(|error| {
                panic!(
                    "missing provenance metadata {}: {error}",
                    metadata_path.display()
                )
            });
            assert!(
                metadata.contains("kellyjonbrazil/jc"),
                "{}",
                metadata_path.display()
            );
            assert!(
                metadata.contains("73fa7d5572dd730076723bd6280786bb9101d32f"),
                "{}",
                metadata_path.display()
            );
        }
    }

    assert!(fixture_count > 0, "expected at least one vendored fixture");
}
