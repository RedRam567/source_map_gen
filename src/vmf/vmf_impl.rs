use super::*;
use crate::map::{Side, Solid};
use vmfparser::ast::{Block, Property};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VmfState {
    pub max_solid_id: u32,
    pub max_side_id: u32,
}

impl<'a> ToBlock<String, VmfState, ()> for Solid<'a> {
    fn to_block(&self, state: &mut VmfState) -> Block<String> {
        state.max_solid_id += 1;
        let id = state.max_solid_id;
        Block {
            name: "solid".to_string(),
            props: vec![Property::new("id", id.to_string())],
            blocks: self.sides.iter().map(|x| x.to_block(state)).collect(),
        }
    }
}

impl<'a> ToBlock<String, VmfState, ()> for Side<'a> {
    fn to_block(&self, state: &mut VmfState) -> Block<String> {
        state.max_side_id += 1;
        let id = state.max_side_id;
        let props = vec![
            Property::new("id", id.to_string()),
            Property::new("plane", self.plane.to_string()),
            Property::new("material", self.texture.material.to_string()),
            Property::new("uaxis", self.texture.uaxis.to_string()),
            Property::new("vaxis", self.texture.vaxis.to_string()),
            // rotation and smoothing group not mandatory
            // rotation is just for hammer display, smoothing group defaults to 0 (none)
            // Property::new("rotation", self.rotation.to_string()),
            Property::new("lightmapscale", self.texture.light_scale.to_string()),
            Property::new("smoothing_groups", "0".to_string()),
        ];
        let blocks = vec![]; // TODO: disp info
        Block { name: "side".to_string(), props, blocks }
    }
}
