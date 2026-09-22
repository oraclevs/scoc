use serde_json::json;

#[test]
fn docker_ps_human_table_is_structured_by_header() {
    let input = br#"CONTAINER ID   IMAGE          COMMAND                  CREATED        STATUS        PORTS                  NAMES
abc123         nginx:latest   \"/docker-entrypoint\"     2 hours ago    Up 2 hours    0.0.0.0:80->80/tcp     web
"#;
    let value = scoc::parse("docker-ps", input, &Default::default()).unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["container_id"], json!("abc123"));
    assert_eq!(rows[0]["image"], json!("nginx:latest"));
    assert_eq!(rows[0]["names"], json!("web"));
}

#[test]
fn docker_stats_derives_numeric_percent_and_byte_fields() {
    let input = b"CONTAINER ID   NAME    CPU %     MEM USAGE / LIMIT      MEM %     NET I/O        BLOCK I/O      PIDS
abc123         web     1.50%     10MiB / 100MiB         10.00%    1kB / 2kB      3MB / 4MB      7
";
    let value = scoc::parse("docker-stats", input, &Default::default()).unwrap();
    let row = &value.as_array().unwrap()[0];
    assert_eq!(row["cpu_percent"], json!(1.5));
    assert_eq!(row["memory_used_bytes"], json!(10 * 1024 * 1024));
    assert_eq!(row["memory_limit_bytes"], json!(100 * 1024 * 1024));
    assert_eq!(row["network_input_bytes"], json!(1024));
    assert_eq!(row["network_output_bytes"], json!(2048));
    assert_eq!(row["pids"], json!(7));
}

#[test]
fn cargo_tree_keeps_depth_relationship() {
    let input =
        "app v0.1.0\n├── serde v1.0.0\n│   └── serde_core v1.0.0\n└── regex v1.0.0\n".as_bytes();
    let value = scoc::parse("cargo-tree", input, &Default::default()).unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows[0]["depth"], json!(0));
    assert_eq!(rows[1]["depth"], json!(1));
    assert!(rows[2]["depth"].as_u64().unwrap() >= 2);
}

#[test]
fn pip_freeze_handles_simple_and_direct_references() {
    let input = b"requests==2.32.5\nmy-lib @ git+https://github.com/example/my-lib.git@abc123\n-e git+https://github.com/example/editable.git#egg=editable\n";
    let value = scoc::parse("pip-freeze", input, &Default::default()).unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["name"], json!("requests"));
    assert_eq!(rows[0]["version"], json!("2.32.5"));
    assert_eq!(
        rows[1]["requirement"],
        json!("my-lib @ git+https://github.com/example/my-lib.git@abc123")
    );
}

#[test]
fn go_env_parses_shell_style_key_values() {
    let input = b"GOARCH='amd64'\nGOOS='linux'\nCGO_ENABLED='1'\n";
    let value = scoc::parse("go-env", input, &Default::default()).unwrap();
    assert_eq!(value["goarch"], json!("amd64"));
    assert_eq!(value["goos"], json!("linux"));
    assert_eq!(value["cgo_enabled"], json!(1));
}

#[test]
fn terraform_workspace_list_preserves_current_marker() {
    let input = b"  default\n* production\n  staging\n";
    let value = scoc::parse("terraform-workspace-list", input, &Default::default()).unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows[1]["name"], json!("production"));
    assert_eq!(rows[1]["current"], json!(true));
}

#[test]
fn buffered_stream_alias_is_chunk_invariant() {
    let input = b"name,age\nAda,36\nLinus,56\n";
    let batch = scoc::parse("csv", input, &Default::default()).unwrap();
    let mut stream = scoc::stream_parser("csv-s", &Default::default()).unwrap();
    let mut got = Vec::new();
    got.extend(stream.push(&input[..5]).unwrap());
    got.extend(stream.push(&input[5..17]).unwrap());
    got.extend(stream.push(&input[17..]).unwrap());
    got.extend(stream.finish().unwrap());
    assert_eq!(json!(got), batch);
}

#[test]
fn dynamic_table_handles_unicode_values_without_byte_offset_panics() {
    let input = "CONTAINER ID   IMAGE          NAMES\nabc123         ñginx:latest   wéb\n";
    let value = scoc::parse("docker-ps", input.as_bytes(), &Default::default()).unwrap();
    let row = &value.as_array().unwrap()[0];
    assert_eq!(row["container_id"], json!("abc123"));
    assert_eq!(row["image"], json!("ñginx:latest"));
    assert_eq!(row["names"], json!("wéb"));
}
