use scoc::{OptionValue, ParseOptions};

// Exact JC 1.26.0 tests/fixtures/ubuntu-18.04/uptime.out.
const INPUT: &[u8] = b" 19:43:06 up 2 days, 19:32,  2 users,  load average: 0.00, 0.00, 0.00\n";

#[test]
fn uptime_adds_derived_fields() {
    let value = scoc::parse("uptime", INPUT, &Default::default()).unwrap();
    assert_eq!(value["time"], "19:43:06");
    assert_eq!(value["load_1m"], 0.0);
    assert_eq!(value["time_hour"], 19);
    assert_eq!(value["time_second"], 6);
    assert_eq!(value["uptime_days"], 2);
    assert_eq!(value["uptime_hours"], 19);
    assert_eq!(value["uptime_minutes"], 32);
}

#[test]
fn uptime_raw_omits_derived_fields() {
    let options = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("uptime", INPUT, &options).unwrap();
    assert_eq!(value["load_1m"], "0.00");
    assert!(value.get("uptime_total_seconds").is_none());
}

#[test]
fn uptime_malformed_input_is_error() {
    let result = scoc::parse("uptime", b"11:35 nonsense\n", &Default::default());
    assert!(matches!(result, Err(scoc::ScocError::Parse { .. })));
}
