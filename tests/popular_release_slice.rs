const EXPECTED_POPULAR_PARSERS: &[&str] = &[
    "apt-cache-show",
    "arp",
    "blkid",
    "chage",
    "cksum",
    "crontab",
    "curl-head",
    "date",
    "df",
    "dig",
    "dmidecode",
    "dpkg-l",
    "du",
    "env",
    "ethtool",
    "file",
    "find",
    "findmnt",
    "free",
    "fstab",
    "getfacl",
    "git-diff",
    "git-log",
    "git-ls-remote",
    "group",
    "hashsum",
    "history",
    "host",
    "hosts",
    "id",
    "ifconfig",
    "ip-route",
    "iptables",
    "jobs",
    "ldd",
    "ls",
    "lsattr",
    "lsb-release",
    "lsblk",
    "lsmod",
    "lsof",
    "lspci",
    "mount",
    "netstat",
    "os-release",
    "pacman",
    "passwd",
    "ping",
    "ps",
    "route",
    "shadow",
    "ss",
    "stat",
    "swapon",
    "systemctl",
    "timedatectl",
    "uname",
    "uptime",
    "w",
    "wc",
    "who",
];

#[test]
fn popular_release_slice_is_registered() {
    for name in EXPECTED_POPULAR_PARSERS {
        assert!(
            scoc::parser(name).is_some(),
            "expected popular parser `{name}` to be registered"
        );
    }
}

#[test]
fn popular_release_slice_has_at_least_sixty_canonical_parsers() {
    let canonical = scoc::parsers()
        .map(|descriptor| descriptor.name)
        .collect::<Vec<_>>();
    assert!(
        canonical.len() >= 60,
        "release slice must expose at least 60 canonical parsers; got {}",
        canonical.len()
    );
}

#[test]
fn popular_release_slice_has_unique_canonical_names() {
    let mut names = scoc::parsers()
        .map(|descriptor| descriptor.name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    let original_len = names.len();
    names.dedup();
    assert_eq!(
        names.len(),
        original_len,
        "canonical parser names must be unique"
    );
}

#[test]
fn windows_only_parsers_are_not_registered() {
    for name in ["dir", "systeminfo", "tasklist", "win-dns-cache", "wmic"] {
        assert!(
            scoc::parser(name).is_none(),
            "Windows-only parser `{name}` must remain deferred"
        );
    }
}
