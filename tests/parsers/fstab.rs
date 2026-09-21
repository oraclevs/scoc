use serde_json::json;

#[test]
fn fstab_defaults_optional_frequency_and_pass_number() {
    let value = scoc::parse("fstab", b"UUID=x / ext4 defaults\n", &Default::default()).unwrap();
    assert_eq!(value[0]["fs_freq"], 0);
    assert_eq!(value[0]["fs_passno"], 0);
}

#[test]
fn fstab_ignores_blank_and_comment_lines() {
    let value = scoc::parse(
        "fstab",
        b"# comment\n\nUUID=x / ext4 defaults 0 2\n",
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        value,
        json!([{
            "fs_spec":"UUID=x","fs_file":"/","fs_vfstype":"ext4","fs_mntops":"defaults","fs_freq":0,"fs_passno":2
        }])
    );
}

#[test]
fn fstab_short_row_is_a_typed_parse_error() {
    let err = scoc::parse("fstab", b"UUID=x / ext4\n", &Default::default()).unwrap_err();
    assert!(matches!(err, scoc::ScocError::Parse { .. }));
}
