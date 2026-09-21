use scoc::{OptionValue, ParseOptions};

// Exact first rows from JC 1.26.0 tests/fixtures/centos-7.7/du.out.
const INPUT: &[u8] = b"134164\t/usr/bin\n43188\t/usr/sbin\n";

#[test]
fn du_matches_upstream_shape_and_raw_conversion() {
    let value = scoc::parse("du", INPUT, &Default::default()).unwrap();
    assert_eq!(value[0]["size"], 134164);
    assert_eq!(value[0]["name"], "/usr/bin");
    let options = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let raw = scoc::parse("du", INPUT, &options).unwrap();
    assert_eq!(raw[0]["size"], "134164");
}

#[test]
fn du_empty_input_is_empty_table() {
    assert_eq!(
        scoc::parse("du", b"", &Default::default()).unwrap(),
        serde_json::json!([])
    );
}
