use vmf_parser_nom::ast::Property;

/// An entity
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entity<S> {
    // pub classname: String,
    // pub solid: Option<Solid<'a>>,
    // pub origin: Option<Vector3<f32>>,
    pub props: Vec<Property<S, S>>,
}

impl<S> Entity<S> {
    pub fn new(props: Vec<Property<S, S>>) -> Self {
        Self { props }
    }
}
