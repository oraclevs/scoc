pub mod terraform_providers;
pub mod terraform_state_list;
pub mod terraform_workspace_list;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 3] = [
    &terraform_state_list::PARSER,
    &terraform_providers::PARSER,
    &terraform_workspace_list::PARSER,
];
