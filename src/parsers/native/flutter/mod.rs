pub mod dart_pub_deps;
pub mod dart_pub_outdated;
pub mod flutter_devices;
pub mod flutter_doctor;
pub mod flutter_emulators;

use crate::ScocParser;
pub(crate) static BUILTINS: [&'static dyn ScocParser; 5] = [
    &flutter_doctor::PARSER,
    &flutter_devices::PARSER,
    &flutter_emulators::PARSER,
    &dart_pub_deps::PARSER,
    &dart_pub_outdated::PARSER,
];
