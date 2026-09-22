pub mod gradle_dependencies;
pub mod gradle_tasks;
pub mod java_version;
pub mod maven_dependency_tree;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 4] = [
    &java_version::PARSER,
    &gradle_tasks::PARSER,
    &gradle_dependencies::PARSER,
    &maven_dependency_tree::PARSER,
];
