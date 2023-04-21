use vmf_parser_nom::ast::Property;

use super::*;

/// An entity
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entity<K, V> {
    // pub classname: String,
    // pub solid: Option<Solid<'a>>,
    // pub origin: Option<Vector3<f32>>,
    pub props: Vec<Property<K, V>>
}
