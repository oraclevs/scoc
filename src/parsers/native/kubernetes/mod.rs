pub mod kubectl_api_resources;
pub mod kubectl_contexts;
pub mod kubectl_events;
pub mod kubectl_get;
pub mod kubectl_top;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 5] = [
    &kubectl_get::PARSER,
    &kubectl_top::PARSER,
    &kubectl_events::PARSER,
    &kubectl_contexts::PARSER,
    &kubectl_api_resources::PARSER,
];
