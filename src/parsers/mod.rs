pub mod jc;
pub mod native;
pub(crate) mod shared;

pub(crate) use jc::common;
pub use jc::{
    acpi, airport, amixer, apt_cache_show, apt_get_sqq, arp, asciitable, asciitable_m,
    authorized_keys, blkid, bluetoothctl, cbt, cef, certbot, chage, cksum, clf, crontab, crontab_u,
    csv, csv_ih, curl_head, date, datetime_iso, debconf_show, df, dig, dmidecode, dpkg_l, du,
    efibootmgr, email_address, env, ethtool, file, find, findmnt, finger, free, fstab, getfacl,
    git_diff, git_log, git_ls_remote, gpg, group, gshadow, hash, hashsum, hciconfig, history, host,
    hosts, http_headers, id, ifconfig, iftop, ini, ini_dup, iostat, ip_address, ip_route, iptables,
    iw_scan, iwconfig, jar_manifest, jobs, jwt, kv, kv_dup, last, ldd, ls, lsattr, lsb_release,
    lsblk, lsmod, lsof, lspci, lsusb, m3u, mdadm, mount, mpstat, needrestart, netrc, netstat,
    nmcli, nsd_control, ntpq, openvpn, os_prober, os_release, pacman, passwd, path, path_list,
    pci_ids, pgpass, pidstat, ping, pip_list, pip_show, pkg_index_apk, pkg_index_deb, plist,
    postconf, proc, proc_buddyinfo, proc_cmdline, proc_consoles, proc_cpuinfo, proc_crypto,
    proc_devices, proc_diskstats, proc_driver_rtc, proc_filesystems, proc_interrupts, proc_iomem,
    proc_ioports, proc_loadavg, proc_locks, proc_meminfo, proc_modules, proc_mtrr, proc_net_arp,
    proc_net_dev, proc_net_dev_mcast, proc_net_if_inet6, proc_net_igmp, proc_net_igmp6,
    proc_net_ipv6_route, proc_net_netlink, proc_net_netstat, proc_net_packet, proc_net_protocols,
    proc_net_route, proc_net_tcp, proc_net_unix, proc_pagetypeinfo, proc_partitions,
    proc_pid_fdinfo, proc_pid_io, proc_pid_maps, proc_pid_mountinfo, proc_pid_numa_maps,
    proc_pid_smaps, proc_pid_stat, proc_pid_statm, proc_pid_status, proc_slabinfo, proc_softirqs,
    proc_stat, proc_swaps, proc_uptime, proc_version, proc_vmallocinfo, proc_vmstat, proc_zoneinfo,
    ps, resolve_conf, route, rpm_qi, rsync, semver, sfdisk, shadow, srt, ss, ssh_conf, sshd_conf,
    stat, swapon, sysctl, syslog, syslog_bsd, systemctl, systemctl_lj, systemctl_ls, systemctl_luf,
    time, timedatectl, timestamp, toml, top, tracepath, traceroute, tsv, tsv_ih, tune2fs, typeset,
    udevadm, ufw, ufw_appinfo, uname, update_alt_gs, update_alt_q, upower, upsc, uptime, url, ver,
    veracrypt, vmstat, w, wc, wg_show, who, x509_cert, x509_crl, x509_csr, xml, xrandr, yaml,
    zipinfo, zpool_iostat, zpool_status,
};

use crate::ScocParser;

pub(crate) fn builtins() -> Vec<&'static dyn ScocParser> {
    let mut out = Vec::with_capacity(274);
    out.extend_from_slice(&jc::BUILTINS);
    out.extend(native::builtins());
    out
}
