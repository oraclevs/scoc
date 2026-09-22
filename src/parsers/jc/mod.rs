pub mod acpi;
pub mod airport;
pub mod amixer;
pub mod apt_cache_show;
pub mod apt_get_sqq;
pub mod arp;
pub mod asciitable;
pub mod asciitable_m;
pub mod authorized_keys;
pub mod blkid;
pub mod bluetoothctl;
pub mod cbt;
pub mod cef;
pub mod certbot;
pub mod chage;
pub mod cksum;
pub mod clf;
pub(crate) mod common;
pub mod crontab;
pub mod crontab_u;
pub mod csv;
pub mod csv_ih;
pub mod curl_head;
pub mod date;
pub mod datetime_iso;
pub mod debconf_show;
pub mod df;
pub mod dig;
pub mod dmidecode;
pub mod dpkg_l;
pub mod du;
pub mod efibootmgr;
pub mod email_address;
pub mod env;
pub mod ethtool;
pub mod file;
pub mod find;
pub mod findmnt;
pub mod finger;
pub mod free;
pub mod fstab;
pub mod getfacl;
pub mod git_diff;
pub mod git_log;
pub mod git_ls_remote;
pub mod gpg;
pub mod group;
pub mod gshadow;
pub mod hash;
pub mod hashsum;
pub mod hciconfig;
pub mod history;
pub mod host;
pub mod hosts;
pub mod http_headers;
pub mod id;
pub mod ifconfig;
pub mod iftop;
pub mod ini;
pub mod ini_dup;
pub mod iostat;
pub mod ip_address;
pub mod ip_route;
pub mod iptables;
pub mod iw_scan;
pub mod iwconfig;
pub mod jar_manifest;
pub mod jobs;
pub mod jwt;
pub mod kv;
pub mod kv_dup;
pub mod last;
pub mod ldd;
pub mod ls;
pub mod lsattr;
pub mod lsb_release;
pub mod lsblk;
pub mod lsmod;
pub mod lsof;
pub mod lspci;
pub mod lsusb;
pub mod m3u;
pub mod mdadm;
pub mod mount;
pub mod mpstat;
pub mod needrestart;
pub mod netrc;
pub mod netstat;
pub mod nmcli;
pub mod nsd_control;
pub mod ntpq;
pub mod openvpn;
pub mod os_prober;
pub mod os_release;
pub mod pacman;
pub mod passwd;
pub mod path;
pub mod path_list;
pub mod pci_ids;
pub mod pgpass;
pub mod pidstat;
pub mod ping;
pub mod pip_list;
pub mod pip_show;
pub mod pkg_index_apk;
pub mod pkg_index_deb;
pub mod plist;
pub mod postconf;
pub mod proc;
pub mod proc_buddyinfo;
pub mod proc_cmdline;
pub mod proc_consoles;
pub mod proc_cpuinfo;
pub mod proc_crypto;
pub mod proc_devices;
pub mod proc_diskstats;
pub mod proc_driver_rtc;
pub mod proc_filesystems;
pub mod proc_interrupts;
pub mod proc_iomem;
pub mod proc_ioports;
pub mod proc_loadavg;
pub mod proc_locks;
pub mod proc_meminfo;
pub mod proc_modules;
pub mod proc_mtrr;
pub mod proc_net_arp;
pub mod proc_net_dev;
pub mod proc_net_dev_mcast;
pub mod proc_net_if_inet6;
pub mod proc_net_igmp;
pub mod proc_net_igmp6;
pub mod proc_net_ipv6_route;
pub mod proc_net_netlink;
pub mod proc_net_netstat;
pub mod proc_net_packet;
pub mod proc_net_protocols;
pub mod proc_net_route;
pub mod proc_net_tcp;
pub mod proc_net_unix;
pub mod proc_pagetypeinfo;
pub mod proc_partitions;
pub mod proc_pid_fdinfo;
pub mod proc_pid_io;
pub mod proc_pid_maps;
pub mod proc_pid_mountinfo;
pub mod proc_pid_numa_maps;
pub mod proc_pid_smaps;
pub mod proc_pid_stat;
pub mod proc_pid_statm;
pub mod proc_pid_status;
pub mod proc_slabinfo;
pub mod proc_softirqs;
pub mod proc_stat;
pub mod proc_swaps;
pub mod proc_uptime;
pub mod proc_version;
pub mod proc_vmallocinfo;
pub mod proc_vmstat;
pub mod proc_zoneinfo;
pub mod ps;
pub mod resolve_conf;
pub mod route;
pub mod rpm_qi;
pub mod rsync;
pub mod semver;
pub mod sfdisk;
pub mod shadow;
pub mod srt;
pub mod ss;
pub mod ssh_conf;
pub mod sshd_conf;
pub mod stat;
pub mod swapon;
pub mod sysctl;
pub mod syslog;
pub mod syslog_bsd;
pub mod systemctl;
pub mod systemctl_lj;
pub mod systemctl_ls;
pub mod systemctl_luf;
pub mod time;
pub mod timedatectl;
pub mod timestamp;
pub mod toml;
pub mod top;
pub mod tracepath;
pub mod traceroute;
pub mod tsv;
pub mod tsv_ih;
pub mod tune2fs;
pub mod typeset;
pub mod udevadm;
pub mod ufw;
pub mod ufw_appinfo;
pub mod uname;
pub mod update_alt_gs;
pub mod update_alt_q;
pub mod upower;
pub mod upsc;
pub mod uptime;
pub mod url;
pub mod ver;
pub mod veracrypt;
pub mod vmstat;
pub mod w;
pub mod wc;
pub mod wg_show;
pub mod who;
pub mod x509_cert;
pub mod x509_crl;
pub mod x509_csr;
pub mod xml;
pub mod xrandr;
pub mod yaml;
pub mod zipinfo;
pub mod zpool_iostat;
pub mod zpool_status;

use crate::ScocParser;

pub(crate) static BUILTINS: [&'static dyn ScocParser; 217] = [
    &acpi::PARSER,
    &airport::PARSER,
    &amixer::PARSER,
    &apt_cache_show::APT_CACHE_SHOW,
    &apt_get_sqq::PARSER,
    &arp::ARP,
    &asciitable::PARSER,
    &asciitable_m::PARSER,
    &authorized_keys::PARSER,
    &blkid::BLKID,
    &bluetoothctl::PARSER,
    &cbt::PARSER,
    &cef::PARSER,
    &certbot::PARSER,
    &chage::CHAGE,
    &cksum::CKSUM,
    &clf::PARSER,
    &crontab::CRONTAB,
    &crontab_u::PARSER,
    &csv::PARSER,
    &csv_ih::PARSER,
    &curl_head::CURL_HEAD,
    &date::DATE,
    &datetime_iso::PARSER,
    &debconf_show::PARSER,
    &df::DF,
    &dig::DIG,
    &dmidecode::DMIDECODE,
    &dpkg_l::DPKG_L,
    &du::DU,
    &efibootmgr::PARSER,
    &email_address::PARSER,
    &env::ENV,
    &ethtool::ETHTOOL,
    &file::FILE,
    &find::FIND,
    &findmnt::FINDMNT,
    &finger::PARSER,
    &free::FREE,
    &fstab::FSTAB,
    &getfacl::GETFACL,
    &git_diff::GIT_DIFF,
    &git_log::GIT_LOG,
    &git_ls_remote::GIT_LS_REMOTE,
    &gpg::PARSER,
    &group::GROUP,
    &gshadow::PARSER,
    &hash::PARSER,
    &hashsum::HASHSUM,
    &hciconfig::PARSER,
    &history::HISTORY,
    &host::HOST,
    &hosts::HOSTS,
    &http_headers::PARSER,
    &id::ID,
    &ifconfig::IFCONFIG,
    &iftop::PARSER,
    &ini::PARSER,
    &ini_dup::PARSER,
    &iostat::PARSER,
    &ip_address::PARSER,
    &ip_route::IP_ROUTE,
    &iptables::IPTABLES,
    &iw_scan::PARSER,
    &iwconfig::PARSER,
    &jar_manifest::PARSER,
    &jobs::JOBS,
    &jwt::PARSER,
    &kv::PARSER,
    &kv_dup::PARSER,
    &last::PARSER,
    &ldd::LDD,
    &ls::LS,
    &lsattr::LSATTR,
    &lsb_release::LSB_RELEASE,
    &lsblk::LSBLK,
    &lsmod::LSMOD,
    &lsof::LSOF,
    &lspci::LSPCI,
    &lsusb::PARSER,
    &m3u::PARSER,
    &mdadm::PARSER,
    &mount::MOUNT,
    &mpstat::PARSER,
    &needrestart::PARSER,
    &netrc::PARSER,
    &netstat::NETSTAT,
    &nmcli::PARSER,
    &nsd_control::PARSER,
    &ntpq::PARSER,
    &openvpn::PARSER,
    &os_prober::PARSER,
    &os_release::OS_RELEASE,
    &pacman::PACMAN,
    &passwd::PASSWD,
    &path::PARSER,
    &path_list::PARSER,
    &pci_ids::PARSER,
    &pgpass::PARSER,
    &pidstat::PARSER,
    &ping::PING,
    &pip_list::PARSER,
    &pip_show::PARSER,
    &pkg_index_apk::PARSER,
    &pkg_index_deb::PARSER,
    &plist::PARSER,
    &postconf::PARSER,
    &proc::PARSER,
    &proc_buddyinfo::PARSER,
    &proc_cmdline::PARSER,
    &proc_consoles::PARSER,
    &proc_cpuinfo::PARSER,
    &proc_crypto::PARSER,
    &proc_devices::PARSER,
    &proc_diskstats::PARSER,
    &proc_driver_rtc::PARSER,
    &proc_filesystems::PARSER,
    &proc_interrupts::PARSER,
    &proc_iomem::PARSER,
    &proc_ioports::PARSER,
    &proc_loadavg::PARSER,
    &proc_locks::PARSER,
    &proc_meminfo::PARSER,
    &proc_modules::PARSER,
    &proc_mtrr::PARSER,
    &proc_net_arp::PARSER,
    &proc_net_dev::PARSER,
    &proc_net_dev_mcast::PARSER,
    &proc_net_if_inet6::PARSER,
    &proc_net_igmp::PARSER,
    &proc_net_igmp6::PARSER,
    &proc_net_ipv6_route::PARSER,
    &proc_net_netlink::PARSER,
    &proc_net_netstat::PARSER,
    &proc_net_packet::PARSER,
    &proc_net_protocols::PARSER,
    &proc_net_route::PARSER,
    &proc_net_tcp::PARSER,
    &proc_net_unix::PARSER,
    &proc_pagetypeinfo::PARSER,
    &proc_partitions::PARSER,
    &proc_pid_fdinfo::PARSER,
    &proc_pid_io::PARSER,
    &proc_pid_maps::PARSER,
    &proc_pid_mountinfo::PARSER,
    &proc_pid_numa_maps::PARSER,
    &proc_pid_smaps::PARSER,
    &proc_pid_stat::PARSER,
    &proc_pid_statm::PARSER,
    &proc_pid_status::PARSER,
    &proc_slabinfo::PARSER,
    &proc_softirqs::PARSER,
    &proc_stat::PARSER,
    &proc_swaps::PARSER,
    &proc_uptime::PARSER,
    &proc_version::PARSER,
    &proc_vmallocinfo::PARSER,
    &proc_vmstat::PARSER,
    &proc_zoneinfo::PARSER,
    &ps::PS,
    &resolve_conf::PARSER,
    &route::ROUTE,
    &rpm_qi::PARSER,
    &rsync::PARSER,
    &semver::PARSER,
    &sfdisk::PARSER,
    &shadow::SHADOW,
    &srt::PARSER,
    &ss::SS,
    &ssh_conf::PARSER,
    &sshd_conf::PARSER,
    &stat::STAT,
    &swapon::SWAPON,
    &sysctl::PARSER,
    &syslog::PARSER,
    &syslog_bsd::PARSER,
    &systemctl::SYSTEMCTL,
    &systemctl_lj::PARSER,
    &systemctl_ls::PARSER,
    &systemctl_luf::PARSER,
    &time::PARSER,
    &timedatectl::TIMEDATECTL,
    &timestamp::PARSER,
    &toml::PARSER,
    &top::PARSER,
    &tracepath::PARSER,
    &traceroute::PARSER,
    &tsv::PARSER,
    &tsv_ih::PARSER,
    &tune2fs::PARSER,
    &typeset::PARSER,
    &udevadm::PARSER,
    &ufw::PARSER,
    &ufw_appinfo::PARSER,
    &uname::UNAME,
    &update_alt_gs::PARSER,
    &update_alt_q::PARSER,
    &upower::PARSER,
    &upsc::PARSER,
    &uptime::UPTIME,
    &url::PARSER,
    &ver::PARSER,
    &veracrypt::PARSER,
    &vmstat::PARSER,
    &w::W,
    &wc::WC,
    &wg_show::PARSER,
    &who::WHO,
    &x509_cert::PARSER,
    &x509_crl::PARSER,
    &x509_csr::PARSER,
    &xml::PARSER,
    &xrandr::PARSER,
    &yaml::PARSER,
    &zipinfo::PARSER,
    &zpool_iostat::PARSER,
    &zpool_status::PARSER,
];
