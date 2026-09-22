pub mod npm_ls;
pub mod npm_outdated;
pub mod npm_run;
pub mod pnpm_list;
pub mod pnpm_outdated;
pub mod yarn_list;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 6] = [
    &npm_run::PARSER,
    &npm_ls::PARSER,
    &npm_outdated::PARSER,
    &pnpm_list::PARSER,
    &pnpm_outdated::PARSER,
    &yarn_list::PARSER,
];
