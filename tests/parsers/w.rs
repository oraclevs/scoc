use scoc::{OptionValue, ParseOptions};

// Exact JC 1.26.0 tests/fixtures/ubuntu-18.04/w.out.
const INPUT: &[u8] = b" 19:43:06 up 2 days, 19:32,  2 users,  load average: 0.00, 0.00, 0.00\nUSER     TTY      FROM             LOGIN@   IDLE   JCPU   PCPU WHAT\nkbrazil  ttyS0    -                21Oct19 14:32  18.09s 18.01s -bash\nkbrazil  pts/0    192.168.71.1     Thu22   10.00s  0.17s  0.00s w\n";

#[test]
fn w_normalizes_dash_to_null() {
    let value = scoc::parse("w", INPUT, &Default::default()).unwrap();
    assert_eq!(value[0]["user"], "kbrazil");
    assert_eq!(value[0]["from"], serde_json::Value::Null);
    assert_eq!(value[0]["login_at"], "21Oct19");
}

#[test]
fn w_raw_keeps_dash() {
    let options = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("w", INPUT, &options).unwrap();
    assert_eq!(value[0]["from"], "-");
}
