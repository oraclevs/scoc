#[test]
fn ls_simple_output_becomes_filename_records() {
    let value = scoc::parse("ls", b"alpha\nbeta file\n", &Default::default()).unwrap();
    assert_eq!(value[0]["filename"], "alpha");
    assert_eq!(value[1]["filename"], "beta file");
}

#[test]
fn ls_symlink_splits_filename_and_target() {
    let input = b"lrwxrwxrwx 1 root root 6 Aug 15 10:53 apropos -> whatis\n";
    let value = scoc::parse("ls", input, &Default::default()).unwrap();
    assert_eq!(value[0]["filename"], "apropos");
    assert_eq!(value[0]["link_to"], "whatis");
    assert_eq!(value[0]["links"], 1);
    assert_eq!(value[0]["size"], 6);
}

#[test]
fn ls_device_rows_use_major_and_minor_instead_of_size() {
    let input = b"crw-rw-rw- 1 root root 1, 3 Sep 16 08:00 null\n";
    let value = scoc::parse("ls", input, &Default::default()).unwrap();
    assert_eq!(value[0]["major_number"], 1);
    assert_eq!(value[0]["minor_number"], 3);
    assert!(value[0].get("size").is_none());
}
