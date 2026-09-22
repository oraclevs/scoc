pub mod docker_compose_images;
pub mod docker_compose_ls;
pub mod docker_compose_ps;
pub mod docker_context_ls;
pub mod docker_images;
pub mod docker_info;
pub mod docker_network_ls;
pub mod docker_ps;
pub mod docker_stats;
pub mod docker_system_df;
pub mod docker_volume_ls;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 11] = [
    &docker_ps::PARSER,
    &docker_images::PARSER,
    &docker_stats::PARSER,
    &docker_network_ls::PARSER,
    &docker_volume_ls::PARSER,
    &docker_system_df::PARSER,
    &docker_info::PARSER,
    &docker_context_ls::PARSER,
    &docker_compose_ps::PARSER,
    &docker_compose_ls::PARSER,
    &docker_compose_images::PARSER,
];
