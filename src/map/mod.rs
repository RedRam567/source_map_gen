mod entity;
mod solid;

pub use entity::*;
pub use solid::*;

use std::fmt::Display;
// use vmfparser::ast::Property;

/// The entire world, consiting of [`Solid`]s, [`Entity`]s, and global info
#[derive(Clone, Debug, PartialEq)]
pub struct Map<'a> {
    pub info: Vec<Prop>,
    pub solids: Vec<Solid<'a>>,
    pub entities: Vec<Entity<'a>>,
}

// TODO: hashmap
/// A property for a map.
#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    key: String,
    value: Value,
}

/// A number or string
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(v) => v.fmt(f),
            Value::Int(v) => v.fmt(f),
            Value::String(v) => v.fmt(f),
        }
    }
}

// impl From<Prop> for Property<String> {
//     fn from(value: Prop) -> Self {
//         Property { key: value.key, value: value.value.to_string() }
//     }
// }
