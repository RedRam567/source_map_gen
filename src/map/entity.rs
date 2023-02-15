use super::*;

/// An entity
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub classname: String,
    pub solid: Option<Solid>,
    pub origin: Option<Point<f64>>,
}
