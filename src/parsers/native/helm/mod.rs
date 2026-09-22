pub mod helm_history;
pub mod helm_list;
pub mod helm_status;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 3] = [
    &helm_list::PARSER,
    &helm_history::PARSER,
    &helm_status::PARSER,
];
