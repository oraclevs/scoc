use scoc::{OptionValue, ParseOptions};

// Exact JC 1.26.0 tests/fixtures/centos-7.7/id.out.
const INPUT: &[u8] = b"uid=1000(kbrazil) gid=1000(kbrazil) groups=1000(kbrazil),10(wheel) context=unconfined_u:unconfined_r:unconfined_t:s0-s0:c0.c1023\n";

#[test]
fn id_matches_jc_nested_shape() {
    let value = scoc::parse("id", INPUT, &Default::default()).unwrap();
    assert_eq!(value["uid"]["id"], 1000);
    assert_eq!(value["uid"]["name"], "kbrazil");
    assert_eq!(value["groups"][1]["id"], 10);
    assert_eq!(value["context"]["role"], "unconfined_r");
}

#[test]
fn id_raw_keeps_ids_as_strings() {
    let options = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("id", INPUT, &options).unwrap();
    assert_eq!(value["uid"]["id"], "1000");
}
