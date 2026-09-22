use std::sync::OnceLock;

use crate::catalog_generated::{native_ecosystem, variants};
use crate::{parser, registry, ParserDescriptor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserOrigin {
    Jc,
    ScocNative { ecosystem: &'static str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationState {
    Unported,
    Implemented,
    RustVerified,
    FixtureVerified,
    DifferentialVerified,
    GoldenVerified,
    Compatible,
    CompatibleWithDeviation,
    Ready,
    DeferredWindows,
}

#[derive(Clone, Copy, Debug)]
pub struct ParserCatalogEntry {
    pub descriptor: &'static ParserDescriptor,
    pub origin: ParserOrigin,
    pub variants: &'static [&'static str],
    pub verification: VerificationState,
}

fn entries() -> &'static Vec<ParserCatalogEntry> {
    static ENTRIES: OnceLock<Vec<ParserCatalogEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        registry()
            .descriptors()
            .map(|descriptor| {
                let origin = if descriptor.upstream.is_some() {
                    ParserOrigin::Jc
                } else {
                    ParserOrigin::ScocNative {
                        ecosystem: native_ecosystem(descriptor.name).unwrap_or("native"),
                    }
                };
                ParserCatalogEntry {
                    descriptor,
                    origin,
                    variants: variants(descriptor.name),
                    verification: VerificationState::Implemented,
                }
            })
            .collect()
    })
}

pub fn catalog() -> impl Iterator<Item = &'static ParserCatalogEntry> {
    entries().iter()
}

pub fn catalog_entry(name: &str) -> Option<&'static ParserCatalogEntry> {
    let canonical = parser(name)?.name;
    entries()
        .iter()
        .find(|entry| entry.descriptor.name == canonical)
}
