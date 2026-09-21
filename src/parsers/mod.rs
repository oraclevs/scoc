pub mod apt_cache_show;
pub mod arp;
pub mod blkid;
pub mod chage;
pub mod cksum;
pub(crate) mod common;
pub mod crontab;
pub mod curl_head;
pub mod date;
pub mod df;
pub mod dig;
pub mod dmidecode;
pub mod dpkg_l;
pub mod du;
pub mod env;
pub mod ethtool;
pub mod file;
pub mod find;
pub mod findmnt;
pub mod free;
pub mod fstab;
pub mod getfacl;
pub mod git_diff;
pub mod git_log;
pub mod git_ls_remote;
pub mod group;
pub mod hashsum;
pub mod history;
pub mod host;
pub mod hosts;
pub mod id;
pub mod ifconfig;
pub mod ip_route;
pub mod iptables;
pub mod jobs;
pub mod ldd;
pub mod ls;
pub mod lsattr;
pub mod lsb_release;
pub mod lsblk;
pub mod lsmod;
pub mod lsof;
pub mod lspci;
pub mod mount;
pub mod netstat;
pub mod os_release;
pub mod pacman;
pub mod passwd;
pub mod ping;
pub mod ps;
pub mod route;
pub mod shadow;
pub mod ss;
pub mod stat;
pub mod swapon;
pub mod systemctl;
pub mod timedatectl;
pub mod uname;
pub mod uptime;
pub mod w;
pub mod wc;
pub mod who;

use crate::ScocParser;

pub(crate) static BUILTINS: [&'static dyn ScocParser; 61] = [
    &apt_cache_show::APT_CACHE_SHOW,
    &arp::ARP,
    &blkid::BLKID,
    &chage::CHAGE,
    &cksum::CKSUM,
    &crontab::CRONTAB,
    &curl_head::CURL_HEAD,
    &date::DATE,
    &df::DF,
    &dig::DIG,
    &dmidecode::DMIDECODE,
    &dpkg_l::DPKG_L,
    &du::DU,
    &env::ENV,
    &ethtool::ETHTOOL,
    &file::FILE,
    &find::FIND,
    &findmnt::FINDMNT,
    &free::FREE,
    &fstab::FSTAB,
    &getfacl::GETFACL,
    &git_diff::GIT_DIFF,
    &git_log::GIT_LOG,
    &git_ls_remote::GIT_LS_REMOTE,
    &group::GROUP,
    &hashsum::HASHSUM,
    &history::HISTORY,
    &host::HOST,
    &hosts::HOSTS,
    &id::ID,
    &ifconfig::IFCONFIG,
    &ip_route::IP_ROUTE,
    &iptables::IPTABLES,
    &jobs::JOBS,
    &ldd::LDD,
    &ls::LS,
    &lsattr::LSATTR,
    &lsb_release::LSB_RELEASE,
    &lsblk::LSBLK,
    &lsmod::LSMOD,
    &lsof::LSOF,
    &lspci::LSPCI,
    &mount::MOUNT,
    &netstat::NETSTAT,
    &os_release::OS_RELEASE,
    &pacman::PACMAN,
    &passwd::PASSWD,
    &ping::PING,
    &ps::PS,
    &route::ROUTE,
    &shadow::SHADOW,
    &ss::SS,
    &stat::STAT,
    &swapon::SWAPON,
    &systemctl::SYSTEMCTL,
    &timedatectl::TIMEDATECTL,
    &uname::UNAME,
    &uptime::UPTIME,
    &w::W,
    &wc::WC,
    &who::WHO,
];
