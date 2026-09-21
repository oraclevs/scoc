use scoc::{OptionValue, ParseOptions};
use serde_json::json;

#[test]
fn raw_env_is_an_object_but_normalized_env_is_a_table() {
    let input = b"A=1\nB=two=parts\n";
    let normalized = scoc::parse("env", input, &Default::default()).unwrap();
    assert_eq!(
        normalized,
        json!([
            {"name":"A","value":"1"},
            {"name":"B","value":"two=parts"}
        ])
    );

    let raw = scoc::parse(
        "env",
        input,
        &ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]),
    )
    .unwrap();
    assert_eq!(raw, json!({"A":"1","B":"two=parts"}));
}

#[test]
fn env_continuation_lines_attach_to_previous_value() {
    let input = b"A=first\ncontinued\nB=two\n";
    let value = scoc::parse("env", input, &Default::default()).unwrap();
    assert_eq!(value[0]["value"], "first\ncontinued");
}

#[test]
fn env_vendored_fixture_is_parseable() {
    let input = crate::common::fixture("generic/env/basic.out");
    let value = scoc::parse("env", &input, &Default::default()).unwrap();
    assert!(value.as_array().is_some_and(|rows| !rows.is_empty()));
}
