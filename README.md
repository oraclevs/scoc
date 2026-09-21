# SCOC

**SCOC** is the SPA Command Output Converter: a Rust-native library that turns command output into structured JSON-shaped values for Spar/Sparsh.

The current release slice targets JC 1.26.0-compatible behavior for more than 60 high-value Linux/macOS/generic parsers while keeping Windows-only parsers deferred. SCOC is a library, not a standalone shell command.

Current canonical parser set:

```text
apt-cache-show  arp          blkid       chage        cksum
crontab         curl-head    date        df           dig
dmidecode       dpkg-l       du          env          ethtool
file            find         findmnt     free         fstab
getfacl         git-diff     git-log     git-ls-remote group
hashsum         history      host        hosts        id
ifconfig        ip-route     iptables    jobs         ldd
ls              lsattr       lsb-release lsblk        lsmod
lsof            lspci        mount       netstat      os-release
pacman          passwd       ping        ps           route
shadow          ss           stat        swapon       systemctl
timedatectl     uname        uptime      w            wc
who
```

`ping` includes incremental streaming support. Parser names are exposed through the SCOC registry and therefore become available to Spar's `from <name>` / `from scoc::<name>` bridge without changing SCOC's public API.

See `compatibility/parser-matrix.toml` for the verification state. Entries remain `in-progress` until the native Rust suite and the pinned JC differential suite have actually passed on the target platform. The current source snapshot was authored in an environment without a Rust toolchain, so compilation, Clippy, Rust tests, and JC differential parity are intentionally **not** claimed here.
