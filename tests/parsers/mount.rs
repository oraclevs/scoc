#[test]
fn mount_linux_and_macos_shapes_follow_upstream_fixtures() {
    // Exact first rows from JC 1.26.0 Ubuntu 18.04 and macOS 10.14.6 fixtures.
    let linux = b"sysfs on /sys type sysfs (rw,nosuid,nodev,noexec,relatime)\n";
    let value = scoc::parse("mount", linux, &Default::default()).unwrap();
    assert_eq!(value[0]["filesystem"], "sysfs");
    assert_eq!(value[0]["mount_point"], "/sys");
    assert_eq!(value[0]["type"], "sysfs");
    assert_eq!(value[0]["options"][0], "rw");

    let mac = b"/dev/disk1s1 on / (apfs, local, journaled)\n";
    let value = scoc::parse("mount", mac, &Default::default()).unwrap();
    assert_eq!(value[0]["filesystem"], "/dev/disk1s1");
    assert!(value[0].get("type").is_none());
}

#[test]
fn mount_malformed_input_is_typed_error() {
    let result = scoc::parse("mount", b"not mount output\n", &Default::default());
    assert!(matches!(result, Err(scoc::ScocError::Parse { .. })));
}

#[test]
fn mount_preserves_jc_key_order() {
    let value = scoc::parse(
        "mount",
        b"sysfs on /sys type sysfs (rw,nosuid)\n",
        &Default::default(),
    )
    .unwrap();
    let keys: Vec<_> = value[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["filesystem", "mount_point", "type", "options"]);
}
