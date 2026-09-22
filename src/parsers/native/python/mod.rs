pub mod pip_check;
pub mod pip_freeze;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 2] =
    [&pip_freeze::PARSER, &pip_check::PARSER];
