use scoc::{OptionValue, ParseOptions};

#[test]
fn ps_axu_normalizes_percent_fields_and_tty() {
    let input = b"USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND\nroot 1 0.5 1.2 100 50 ? Ss 10:00 0:01 /sbin/init --system\n";
    let value = scoc::parse("ps", input, &Default::default()).unwrap();
    assert_eq!(value[0]["pid"], 1);
    assert_eq!(value[0]["cpu_percent"], 0.5);
    assert_eq!(value[0]["mem_percent"], 1.2);
    assert!(value[0]["tty"].is_null());
    assert_eq!(value[0]["command"], "/sbin/init --system");
}

#[test]
fn ps_raw_keeps_numbers_and_unknown_tty_as_strings() {
    let input = b"USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND\nroot 1 0.5 1.2 100 50 ? Ss 10:00 0:01 init\n";
    let opts = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("ps", input, &opts).unwrap();
    assert_eq!(value[0]["pid"], "1");
    assert_eq!(value[0]["tty"], "?");
}
