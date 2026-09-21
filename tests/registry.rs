#[test]
fn unknown_parser_is_typed_error() {
    let err = scoc::parse("does-not-exist", b"", &Default::default()).unwrap_err();
    assert!(matches!(err, scoc::ScocError::UnknownParser { .. }));
}

#[test]
fn lookup_is_ascii_case_insensitive_but_not_underscore_rewriting() {
    assert!(scoc::parser("DF").is_some());
    assert!(scoc::parser("ping_s").is_none());
}

#[test]
fn forcing_streaming_on_batch_parser_is_error() {
    let result = scoc::stream_parser("env", &Default::default());
    assert!(matches!(
        result,
        Err(scoc::ScocError::StreamingUnsupported { .. })
    ));
}

#[test]
fn all_canonical_descriptor_names_are_unique() {
    let names: Vec<_> = scoc::parsers().map(|p| p.name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(names.len(), sorted.len());
}

#[test]
fn scoc_parser_version_is_distinct_from_upstream_parser_version() {
    let df = scoc::parser("df").expect("df parser");
    assert_eq!(df.parser_version, "0.1.0");
    let upstream = df.upstream.expect("JC metadata");
    assert_eq!(upstream.standard_version, "2.1");
}
