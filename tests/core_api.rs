use scoc::{OptionSpec, OptionValue, OutputShape, ParserOutput};

#[test]
fn compatibility_baseline_is_pinned_to_jc_1_26_0() {
    let baseline = scoc::compatibility_baseline();
    assert_eq!(baseline.project, "jc");
    assert_eq!(baseline.version, "1.26.0");
    assert_eq!(baseline.commit, "73fa7d5572dd730076723bd6280786bb9101d32f");
}

#[test]
fn parser_output_can_describe_raw_shape_changes() {
    let output = ParserOutput {
        normalized: OutputShape::Table,
        raw: Some(OutputShape::Record),
        stream_item: None,
    };
    assert_eq!(output.shape(false), OutputShape::Table);
    assert_eq!(output.shape(true), OutputShape::Record);
}

#[test]
fn bool_option_rejects_a_string_value() {
    let spec = OptionSpec::bool("raw", false, "Return JC raw output");
    let error = spec
        .validate(&OptionValue::String("true".into()))
        .expect_err("string is not a bool");
    assert!(error.to_string().contains("raw"));
    assert!(error.to_string().contains("bool"));
}
