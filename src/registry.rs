use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::parsers::builtins;
use crate::{ParseOptions, ParserDescriptor, ScocError, ScocParser, ScocStreamParser};

pub struct ParserRegistry {
    lookup: HashMap<String, &'static dyn ScocParser>,
    parsers: Vec<&'static dyn ScocParser>,
}

impl ParserRegistry {
    fn from_parsers(parsers: Vec<&'static dyn ScocParser>) -> Result<Self, ScocError> {
        let mut lookup = HashMap::new();
        let mut canonical = HashMap::new();
        for parser in &parsers {
            let descriptor = parser.descriptor();
            let canonical_key = normalize_name(descriptor.name);
            if canonical
                .insert(canonical_key.clone(), descriptor.name)
                .is_some()
            {
                return Err(ScocError::internal(format!(
                    "duplicate canonical parser `{}`",
                    descriptor.name
                )));
            }
            if lookup.insert(canonical_key, *parser).is_some() {
                return Err(ScocError::internal(format!(
                    "registry collision at `{}`",
                    descriptor.name
                )));
            }
            for alias in descriptor.aliases {
                let key = normalize_name(alias);
                if canonical.contains_key(&key) || lookup.contains_key(&key) {
                    return Err(ScocError::internal(format!(
                        "parser alias collision at `{alias}`"
                    )));
                }
                lookup.insert(key, *parser);
            }
        }
        Ok(Self { lookup, parsers })
    }

    fn builtin() -> Result<Self, ScocError> {
        Self::from_parsers(builtins())
    }

    pub fn parser_impl(&self, name: &str) -> Option<&'static dyn ScocParser> {
        self.lookup.get(&normalize_name(name)).copied()
    }

    pub fn descriptor(&self, name: &str) -> Option<&'static ParserDescriptor> {
        self.parser_impl(name).map(|parser| parser.descriptor())
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &'static ParserDescriptor> + '_ {
        self.parsers.iter().map(|parser| parser.descriptor())
    }
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub fn registry() -> &'static ParserRegistry {
    static REGISTRY: OnceLock<ParserRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        ParserRegistry::builtin().expect("SCOC builtin registry must be collision free")
    })
}

pub fn parser(name: &str) -> Option<&'static ParserDescriptor> {
    registry().descriptor(name)
}

pub fn parsers() -> impl Iterator<Item = &'static ParserDescriptor> {
    registry().descriptors()
}

pub fn parse(name: &str, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
    let parser = registry()
        .parser_impl(name)
        .ok_or_else(|| ScocError::UnknownParser {
            name: name.to_string(),
        })?;
    let descriptor = parser.descriptor();
    options.validate_for(descriptor.name, descriptor.options)?;
    parser.parse(input, options)
}

pub fn stream_parser(
    name: &str,
    options: &ParseOptions,
) -> Result<Box<dyn ScocStreamParser>, ScocError> {
    let parser = registry()
        .parser_impl(name)
        .ok_or_else(|| ScocError::UnknownParser {
            name: name.to_string(),
        })?;
    let descriptor = parser.descriptor();
    options.validate_for(descriptor.name, descriptor.options)?;
    if !descriptor.capabilities.streaming {
        return Err(ScocError::StreamingUnsupported {
            parser: descriptor.name.into(),
        });
    }
    parser.stream_parser(options)
}
