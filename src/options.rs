use std::collections::HashMap;

use crate::error::ScocError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionKind {
    Bool,
    Integer,
    Float,
    String,
    Enum(&'static [&'static str]),
}

impl OptionKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Integer => "int",
            Self::Float => "float",
            Self::String => "str",
            Self::Enum(_) => "enum",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionDefault {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(&'static str),
    Enum(&'static str),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OptionValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl From<bool> for OptionValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<i64> for OptionValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}
impl From<f64> for OptionValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<String> for OptionValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<&str> for OptionValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl OptionValue {
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "int",
            Self::Float(_) => "float",
            Self::String(_) => "str",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptionSpec {
    pub name: &'static str,
    pub kind: OptionKind,
    pub required: bool,
    pub default: Option<OptionDefault>,
    pub description: &'static str,
}

impl OptionSpec {
    pub const fn bool(name: &'static str, default: bool, description: &'static str) -> Self {
        Self {
            name,
            kind: OptionKind::Bool,
            required: false,
            default: Some(OptionDefault::Bool(default)),
            description,
        }
    }

    pub const fn integer(
        name: &'static str,
        default: Option<i64>,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            kind: OptionKind::Integer,
            required: false,
            default: match default {
                Some(v) => Some(OptionDefault::Integer(v)),
                None => None,
            },
            description,
        }
    }

    pub const fn string(
        name: &'static str,
        default: Option<&'static str>,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            kind: OptionKind::String,
            required: false,
            default: match default {
                Some(v) => Some(OptionDefault::String(v)),
                None => None,
            },
            description,
        }
    }

    pub fn validate(&self, value: &OptionValue) -> Result<(), ScocError> {
        let valid = match (self.kind, value) {
            (OptionKind::Bool, OptionValue::Bool(_)) => true,
            (OptionKind::Integer, OptionValue::Integer(_)) => true,
            (OptionKind::Float, OptionValue::Float(_) | OptionValue::Integer(_)) => true,
            (OptionKind::String, OptionValue::String(_)) => true,
            (OptionKind::Enum(values), OptionValue::String(value)) => {
                values.contains(&value.as_str())
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(ScocError::InvalidOption {
                parser: "<unbound>".into(),
                option: self.name.into(),
                reason: format!("expected {}, got {}", self.kind.name(), value.kind_name()),
            })
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ParseOptions {
    values: HashMap<String, OptionValue>,
}

impl ParseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs<I, K>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, OptionValue)>,
        K: Into<String>,
    {
        let values = pairs.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Self { values }
    }

    pub fn insert(&mut self, name: impl Into<String>, value: OptionValue) -> Option<OptionValue> {
        self.values.insert(name.into(), value)
    }

    pub fn get(&self, name: &str) -> Option<&OptionValue> {
        self.values.get(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &OptionValue)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn bool(&self, name: &str) -> Option<bool> {
        match self.get(name) {
            Some(OptionValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }
    pub fn integer(&self, name: &str) -> Option<i64> {
        match self.get(name) {
            Some(OptionValue::Integer(v)) => Some(*v),
            _ => None,
        }
    }
    pub fn float(&self, name: &str) -> Option<f64> {
        match self.get(name) {
            Some(OptionValue::Float(v)) => Some(*v),
            Some(OptionValue::Integer(v)) => Some(*v as f64),
            _ => None,
        }
    }
    pub fn string(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(OptionValue::String(v)) => Some(v),
            _ => None,
        }
    }

    pub fn validate_for(&self, parser: &str, specs: &[OptionSpec]) -> Result<(), ScocError> {
        for (name, value) in self.iter() {
            let Some(spec) = specs.iter().find(|spec| spec.name == name) else {
                return Err(ScocError::InvalidOption {
                    parser: parser.into(),
                    option: name.into(),
                    reason: "unknown option".into(),
                });
            };
            if let Err(error) = spec.validate(value) {
                let reason = match error {
                    ScocError::InvalidOption { reason, .. } => reason,
                    other => other.to_string(),
                };
                return Err(ScocError::InvalidOption {
                    parser: parser.into(),
                    option: name.into(),
                    reason,
                });
            }
        }
        for spec in specs.iter().filter(|spec| spec.required) {
            if self.get(spec.name).is_none() {
                return Err(ScocError::InvalidOption {
                    parser: parser.into(),
                    option: spec.name.into(),
                    reason: "required option is missing".into(),
                });
            }
        }
        Ok(())
    }
}
