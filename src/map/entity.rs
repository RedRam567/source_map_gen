use super::*;

/// An entity
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entity<'a> {
    pub classname: String,
    pub solid: Option<Solid<'a>>,
    pub origin: Option<Vector3<f32>>,
}
