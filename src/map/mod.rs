//! One step above vmf blocks and properties. [`Solid`]s and [`Entity`]s. Also [`Map`], a group of `Solids` and `Entities`

pub(crate) mod entity;
pub(crate) mod solid;
pub(crate) mod texture;
pub(crate) mod vector;

pub use entity::*;
pub use solid::*;
pub use texture::*;
pub use vector::*;

use crate::OneOrVec;
use crate::{StrType};
use crate::generation::Bounds;
use vmf_parser_nom::ast::Property;


#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapOptions {
    /// Surround the level with a giant box with skybox textures.
    /// Notoriously bad for compile times and optimization but will prevent leaks.
    /// Good for quick testing.
    /// TODO: add as actual cordon.
    pub cordon: Option<Bounds<f32>>,
    // TODO: skybox, detail texture, name or smth
    pub sky_name: String,
    // TODO: ooo aditional files, nav, missions or smth, pop
}

impl MapOptions {
    pub fn defaults_l4d2(&mut self) -> &mut Self {
        self.sky_name = "sky_l4d_rural02_hdr".to_string();
        self
    }
    pub fn defaults_tf2(&mut self) -> &mut Self {
        self.sky_name = "sky_day01_01".to_string();
        self
    }
}

/// The entire world, consiting of [`Solid`]s, [`Entity`]s, and global info
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Map<'a> {
    pub options: MapOptions,
    pub solids: Vec<Solid<'a>>,
    pub entities: Vec<Entity<StrType<'a>>>,
}

impl<'a> Map<'a> {
    pub fn defaults_l4d2(&mut self) -> &mut Self {
        self.options.sky_name = "sky_l4d_rural02_hdr".to_string();
        self.entities.push(Entity {
            // c1m1_hotel
            props: vec![
                Property::new("origin", "0 0 0"),
                Property::new("SunSpreadAngle", "0"),
                Property::new("pitch", "-14"),
                Property::new("angles", "0 30 0"),
                Property::new("_lightscaleHDR", "1"),
                Property::new("_lightHDR", "-1 -1 -1 1"),
                Property::new("_light", "228 215 192 400"),
                Property::new("_AmbientScaleHDR", "1"),
                Property::new("_ambientHDR", "-1 -1 -1 1"),
                Property::new("_ambient", "171 206 220 50"),
                Property::new("classname", "light_environment"),
            ],
        });
        self
    }
    pub fn defaults_tf2(&mut self) -> &mut Self {
        self.options.sky_name = "sky_day01_01".to_string();
        self
    }
}

impl<'a> Map<'a> {
    pub fn add_solid(&mut self, solid: Solid<'a>) {
        self.solids.push(solid);
    }

    // TODO: better
    /// uses [`OneOrVec`]
    pub fn add_solid2(&mut self, solid: OneOrVec<Solid<'a>>) {
        self.solids.extend(solid.to_vec());
    }
}

// entity
// {
// 	"id" "4408975"
// 	"origin" "1297.04 5176.84 3025"
// 	"SunSpreadAngle" "0"
// 	"pitch" "-14"
// 	"angles" "0 30 0"
// 	"_lightscaleHDR" "1"
// 	"_lightHDR" "-1 -1 -1 1"
// 	"_light" "228 215 192 400"
// 	"_AmbientScaleHDR" "1"
// 	"_ambientHDR" "-1 -1 -1 1"
// 	"_ambient" "171 206 220 50"
// 	"classname" "light_environment"
// }
