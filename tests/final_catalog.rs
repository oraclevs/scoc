#[test]
fn final_registry_has_every_first_class_jc_and_native_parser() {
    let expected_jc = include_str!("../compatibility/expected-first-class-jc.txt")
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let expected_native = include_str!("../compatibility/expected-native.txt")
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(expected_jc.len(), 217);
    assert_eq!(expected_native.len(), 57);
    assert_eq!(scoc::parsers().count(), 274);

    for name in expected_jc.into_iter().chain(expected_native) {
        assert!(scoc::parser(name).is_some(), "missing parser {name}");
    }
}

#[test]
fn windows_only_jc_parsers_are_not_registered() {
    for name in [
        "dir",
        "ipconfig",
        "net-localgroup",
        "net-user",
        "route-print",
        "systeminfo",
    ] {
        assert!(
            scoc::parser(name).is_none(),
            "windows-only parser unexpectedly registered: {name}"
        );
    }
}

#[test]
fn streaming_aliases_resolve_without_inflating_canonical_count() {
    for (canonical, alias) in [
        ("airport", "airport-s"),
        ("cef", "cef-s"),
        ("clf", "clf-s"),
        ("csv", "csv-s"),
        ("csv-ih", "csv-ih-s"),
        ("git-log", "git-log-s"),
        ("iostat", "iostat-s"),
        ("ls", "ls-s"),
        ("mpstat", "mpstat-s"),
        ("pidstat", "pidstat-s"),
        ("ping", "ping-s"),
        ("rsync", "rsync-s"),
        ("stat", "stat-s"),
        ("syslog", "syslog-s"),
        ("syslog-bsd", "syslog-bsd-s"),
        ("top", "top-s"),
        ("traceroute", "traceroute-s"),
        ("tsv", "tsv-s"),
        ("tsv-ih", "tsv-ih-s"),
        ("vmstat", "vmstat-s"),
    ] {
        let canonical_desc = scoc::parser(canonical).expect(canonical);
        let alias_desc = scoc::parser(alias).expect(alias);
        assert_eq!(canonical_desc.name, alias_desc.name);
    }
    assert_eq!(scoc::parsers().count(), 274);
}

#[test]
fn typed_variant_and_internal_errors_exist() {
    assert!(matches!(
        scoc::ScocError::unsupported_variant("docker-ps", "--format custom"),
        scoc::ScocError::UnsupportedVariant { .. }
    ));
    assert!(matches!(
        scoc::ScocError::internal("boom"),
        scoc::ScocError::Internal { .. }
    ));
}

#[test]
fn native_catalog_origins_are_exposed() {
    let docker = scoc::catalog_entry("docker-ps").expect("docker catalog");
    assert_eq!(
        docker.origin,
        scoc::ParserOrigin::ScocNative {
            ecosystem: "docker"
        }
    );
    let df = scoc::catalog_entry("df").expect("df catalog");
    assert_eq!(df.origin, scoc::ParserOrigin::Jc);
}
