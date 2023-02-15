use super::*;

/// An entity
#[derive(Clone, Debug, PartialEq)]
pub struct Entity<'a> {
    pub classname: String,
    pub solid: Option<Solid<'a>>,
    pub origin: Option<Point<f64>>,
}
