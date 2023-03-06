use super::*;
use crate::{
    generation::region::{Room},
    map::{IdInfo, Map, Side, Solid},
};
use vmf_parser_nom::ast::{Block, Property};

impl<'a> ToBlock<String, IdInfo, ()> for Solid<'a> {
    fn to_block(&self, state: &mut IdInfo) -> Block<String> {
        state.max_solid_id += 1;
        let id = state.max_solid_id;
        Block {
            name: "solid".to_string(),
            props: vec![Property::new("id", id.to_string())],
            blocks: self.sides.iter().map(|x| x.to_block(state)).collect(),
        }
    }
}

impl<'a> ToBlock<String, IdInfo, ()> for Side<'a> {
    fn to_block(&self, state: &mut IdInfo) -> Block<String> {
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

impl<'a> ToVmf<String, IdInfo, ()> for Map<'a> {
    fn to_vmf(&self, state: &mut IdInfo) -> Vmf<String> {
        let mut vmf = Vmf::default();
        let mut solids: Vec<_> = self.solids.iter().map(|s| s.to_block(state)).collect();
        if let Some(cordon) = &self.options.dev_skybox {
            solids.extend(
                Room::new(cordon.clone())
                    .construct_sky()
                    .into_iter()
                    .map(|s| s.to_block(state)),
            );
        }
        vmf.push(Block {
            name: "versioninfo".to_string(),
            props: vec![
                Property::new("editorversion", "400".to_string()),
                Property::new("editorbuild", "9540".to_string()),
                Property::new("mapversion", "1".to_string()),
                Property::new("formatversion", "100".to_string()),
                Property::new("prefab", "0".to_string()),
            ],
            blocks: vec![],
        });
        vmf.push(Block { name: "visgroups".to_string(), props: vec![], blocks: vec![] });
        vmf.push(Block {
            name: "viewsettings".to_string(),
            props: vec![
                Property::new("bSnapToGrid", "1".to_string()),
                Property::new("bShowGrid", "1".to_string()),
                Property::new("bShowLogicalGrid", "0".to_string()),
                Property::new("nGridSpacing", "64".to_string()),
                Property::new("bShow3DGrid", "0".to_string()),
            ],
            blocks: vec![],
        });
        vmf.push(Block {
            name: "world".to_string(),
            props: vec![
                Property::new("id", "1".to_string()),
                Property::new("mapversion", "1".to_string()),
                Property::new("classname", "worldspawn".to_string()),
                Property::new("skyname", "sky_day01_01".to_string()),
            ],
            blocks: solids,
        });
        // cameras unnessesary
        // cordons unnessesary
        vmf
    }
}
