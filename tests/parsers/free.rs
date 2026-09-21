use scoc::{OptionValue, ParseOptions};

// Exact JC 1.26.0 tests/fixtures/ubuntu-18.04/free.out.
const INPUT: &[u8] = b"              total        used        free      shared  buff/cache   available\nMem:        2017300      242740      478228        1196     1296332     1585920\nSwap:       2097148         268     2096880\n";

#[test]
fn free_normalizes_types_and_buff_cache() {
    let value = scoc::parse("free", INPUT, &Default::default()).unwrap();
    assert_eq!(value[0]["type"], "Mem");
    assert_eq!(value[0]["total"], 2017300);
    assert_eq!(value[0]["buff_cache"], 1296332);
    assert_eq!(value[1]["type"], "Swap");
}

#[test]
fn free_raw_keeps_numeric_strings() {
    let options = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("free", INPUT, &options).unwrap();
    assert_eq!(value[0]["total"], "2017300");
}
