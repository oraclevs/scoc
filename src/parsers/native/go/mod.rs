pub mod go_env;
pub mod go_mod_graph;
pub mod go_version;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 3] =
    [&go_version::PARSER, &go_env::PARSER, &go_mod_graph::PARSER];
