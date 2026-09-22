pub mod git_branch;
pub mod git_remote;
pub mod git_stash;
pub mod git_status;
pub mod git_submodule;
pub mod git_tag;
pub mod git_worktree;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 7] = [
    &git_status::PARSER,
    &git_branch::PARSER,
    &git_remote::PARSER,
    &git_tag::PARSER,
    &git_stash::PARSER,
    &git_worktree::PARSER,
    &git_submodule::PARSER,
];
