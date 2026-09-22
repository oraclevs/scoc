pub mod gh_issue_list;
pub mod gh_pr_list;
pub mod gh_run_list;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 3] = [
    &gh_pr_list::PARSER,
    &gh_issue_list::PARSER,
    &gh_run_list::PARSER,
];
