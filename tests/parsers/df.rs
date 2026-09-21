use scoc::{OptionValue, ParseOptions};

#[test]
fn df_normalizes_linux_columns_and_percent() {
    let input = b"Filesystem     1K-blocks  Used Available Use% Mounted on\n/dev/root       1024      512   512       50% /\n";
    let value = scoc::parse("df", input, &Default::default()).unwrap();
    assert_eq!(value[0]["filesystem"], "/dev/root");
    assert_eq!(value[0]["1k_blocks"], 1024);
    assert_eq!(value[0]["used"], 512);
    assert_eq!(value[0]["available"], 512);
    assert_eq!(value[0]["use_percent"], 50);
    assert_eq!(value[0]["mounted_on"], "/");
}

#[test]
fn df_human_sizes_convert_to_bytes() {
    let input = b"Filesystem      Size  Used Avail Use% Mounted on\n/dev/root       1G    512M  512M  50% /\n";
    let value = scoc::parse("df", input, &Default::default()).unwrap();
    assert_eq!(value[0]["size"], 1_073_741_824u64);
    assert_eq!(value[0]["used"], 536_870_912u64);
}

#[test]
fn df_raw_preserves_strings() {
    let input = b"Filesystem     1K-blocks  Used Available Use% Mounted on\n/dev/root           1024   512       512  50% /\n";
    let opts = ParseOptions::from_pairs([("raw", OptionValue::Bool(true))]);
    let value = scoc::parse("df", input, &opts).unwrap();
    assert_eq!(value[0]["1k_blocks"], "1024");
    assert_eq!(value[0]["use%"], "50%");
}
