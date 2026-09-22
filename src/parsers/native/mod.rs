pub mod docker;
pub mod flutter;
pub mod git;
pub mod github;
pub mod go;
pub mod helm;
pub mod java;
pub mod javascript;
pub mod kubernetes;
pub mod python;
pub mod rust;
pub mod terraform;

use crate::ScocParser;

pub(crate) fn builtins() -> Vec<&'static dyn ScocParser> {
    let mut out = Vec::with_capacity(57);
    out.extend_from_slice(&docker::BUILTINS);
    out.extend_from_slice(&flutter::BUILTINS);
    out.extend_from_slice(&git::BUILTINS);
    out.extend_from_slice(&github::BUILTINS);
    out.extend_from_slice(&go::BUILTINS);
    out.extend_from_slice(&helm::BUILTINS);
    out.extend_from_slice(&java::BUILTINS);
    out.extend_from_slice(&javascript::BUILTINS);
    out.extend_from_slice(&kubernetes::BUILTINS);
    out.extend_from_slice(&python::BUILTINS);
    out.extend_from_slice(&rust::BUILTINS);
    out.extend_from_slice(&terraform::BUILTINS);
    out
}
