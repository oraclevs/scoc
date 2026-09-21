use scoc::{OptionValue, ParseOptions};

// Exact data row from JC 1.26.0 tests/fixtures/ubuntu-18.04/who-a.out.
const INPUT: &[u8] = b"kbrazil  + pts/0        2020-03-02 02:54   .          2176 (192.168.71.1)\n";

#[test]
fn who_parses_upstream_linux_login_row() {
    let value = scoc::parse("who", INPUT, &Default::default()).unwrap();
    assert_eq!(value[0]["user"], "kbrazil");
    assert_eq!(value[0]["tty"], "pts/0");
    assert_eq!(value[0]["pid"], 2176);
    assert_eq!(value[0]["from"], "192.168.71.1");
    assert!(value[0].get("epoch").is_some());
}

#[test]
fn who_raw_keeps_pid_string_and_omits_epoch() {
    let options = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("who", INPUT, &options).unwrap();
    assert_eq!(value[0]["pid"], "2176");
    assert!(value[0].get("epoch").is_none());
}
