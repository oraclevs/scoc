pub mod cargo_install_list;
pub mod cargo_tree;
pub mod rustc_version;
pub mod rustup_targets;
pub mod rustup_toolchains;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 5] = [
    &cargo_tree::PARSER,
    &cargo_install_list::PARSER,
    &rustup_toolchains::PARSER,
    &rustup_targets::PARSER,
    &rustc_version::PARSER,
];
