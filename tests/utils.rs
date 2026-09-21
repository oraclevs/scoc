use scoc::utils::{parse_human_size, simple_table_parse, sparse_table_parse};
use serde_json::{json, Value};

#[test]
fn simple_table_keeps_spaces_in_last_column() {
    let rows = simple_table_parse(&["uid pid cmd", "root 1 /usr/lib/systemd --system"]).unwrap();
    assert_eq!(rows[0]["cmd"], json!("/usr/lib/systemd --system"));
}

#[test]
fn sparse_table_preserves_blank_cells_as_null() {
    let rows = sparse_table_parse(&[
        "filesystem  size  used  mounted_on",
        "/dev/a      10          /",
    ])
    .unwrap();
    assert_eq!(rows[0]["used"], Value::Null);
}

#[test]
fn human_sizes_match_jc_unit_rules() {
    assert_eq!(parse_human_size("1 KB", false).unwrap(), 1000);
    assert_eq!(parse_human_size("1 KB", true).unwrap(), 1024);
    assert_eq!(parse_human_size("1 KiB", false).unwrap(), 1024);
    assert_eq!(parse_human_size("1K", true).unwrap(), 1024);
    assert_eq!(parse_human_size("1.5 GB", true).unwrap(), 1_610_612_736);
}
