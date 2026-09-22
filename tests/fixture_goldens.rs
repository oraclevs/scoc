use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn checked_in_native_contract_cases_match_public_api() {
    let manifest_path = root().join("tests/fixtures/native/cases.json");
    let cases: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).expect("native cases manifest"))
            .expect("valid native cases json");
    for case in cases.as_array().expect("cases array") {
        let parser = case["parser"].as_str().expect("parser name");
        let input = root().join(Path::new(case["input"].as_str().expect("input path")));
        let expected = root().join(Path::new(case["expected"].as_str().expect("expected path")));
        let expected: Value =
            serde_json::from_slice(&std::fs::read(expected).expect("expected json"))
                .expect("valid expected json");
        let actual = scoc::parse(
            parser,
            &std::fs::read(input).expect("fixture input"),
            &Default::default(),
        )
        .unwrap_or_else(|err| panic!("{parser} failed: {err}"));
        assert_eq!(actual, expected, "fixture mismatch for {parser}");
    }
}
