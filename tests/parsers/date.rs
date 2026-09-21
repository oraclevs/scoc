#[test]
fn date_parses_upstream_12_hour_utc_fixture_shape() {
    // Exact JC 1.26.0 tests/fixtures/ubuntu-20.04/date.out.
    let value = scoc::parse(
        "date",
        b"Tue Jan  5 01:02:04 AM UTC 2021\n",
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value["year"], 2021);
    assert_eq!(value["month"], "Jan");
    assert_eq!(value["month_num"], 1);
    assert_eq!(value["weekday"], "Tue");
    assert_eq!(value["hour"], 1);
    assert_eq!(value["hour_24"], 1);
    assert_eq!(value["timezone"], "UTC");
    assert_eq!(value["utc_offset"], "+0000");
    assert_eq!(value["timezone_aware"], true);
}

#[test]
fn date_malformed_input_is_typed_error() {
    let result = scoc::parse("date", b"not a date\n", &Default::default());
    assert!(matches!(result, Err(scoc::ScocError::Parse { .. })));
}
