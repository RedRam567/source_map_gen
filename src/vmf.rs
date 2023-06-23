//! Vmf format traits and impls.
//! See [`vmf_parser_nom`] crate.

use crate::generation2::disp::Displacement;
use crate::prelude::Texture;
use crate::prelude::UVAxis;
use crate::utils::Vec2d;
use crate::utils::{NextChunk, TryMap};
use crate::{
    generation::region::Room,
    map::{DispInfo, Entity, Map, Side, Solid},
    prelude::{Plane, Vector3},
    StrType,
};
use std::fmt::Display;
use std::str::FromStr;
use vmf_parser_nom::ast::{Block, Property, Vmf};

// TODO: different trait?
// tolower and toblock?
// FIXME: switch almost all to to_lower instead of into

/// Convert to a lower level of abstraction.
/// Example: A [`Solid`](crate::map::solid::Solid) into a [`Block`](vmf_parser_nom::ast::Block).
pub trait ToLower<T>: Clone {
    /// Convert to a lower level of abstraction. See [`ToLower`]
    fn into_lower(self) -> T {
        self.to_lower()
    }

    /// Convert to an owned lower level of abstraction. See [`ToLower`]
    fn to_lower(&self) -> T {
        self.clone().into_lower()
    }
}

/// Convert to a higher level of abstraction.
/// Example: A [`Block`](vmf_parser_nom::ast::Block) into a [`Solid`](crate::map::solid::Solid).
pub(crate) trait ToHigher<T>: Clone {
    /// Convert this into a low level map element, consuming `self`. See [`ToHigher`].
    fn into_higher(self) -> T {
        self.to_higher()
    }
    /// Convert this into an owned low level map element. See [`ToHigher`].
    fn to_higher(&self) -> T {
        self.clone().into_higher()
    }
}

// Implementations //

impl<'a> ToLower<Vmf<StrType<'a>>> for Map<'a> {
    fn into_lower(self) -> Vmf<StrType<'a>> {
        let mut vmf = Vmf::default();
        let mut solid_blocks: Vec<_> = self.solids.iter().map(|s| s.to_lower()).collect();
        if let Some(cordon) = self.options.cordon.clone() {
            // TODO: add as actual cordon
            solid_blocks
                .extend(Room::new(cordon).construct_sky_inside().iter().map(|s| s.to_lower()));
        }

        vmf.inner.blocks.push(Block {
            name: "versioninfo".into(),
            props: vec![
                Property::new("editorversion", "400"),
                Property::new("editorbuild", "9540"),
                Property::new("mapversion", "1"),
                Property::new("formatversion", "100"),
                Property::new("prefab", "0"),
            ],
            blocks: vec![],
        });
        vmf.inner
            .blocks
            .push(Block { name: "visgroups".into(), props: vec![], blocks: vec![] });
        vmf.inner.blocks.push(Block {
            name: "viewsettings".into(),
            props: vec![
                Property::new("bSnapToGrid", "1"),
                Property::new("bShowGrid", "1"),
                Property::new("bShowLogicalGrid", "0"),
                Property::new("nGridSpacing", "64"),
                Property::new("bShow3DGrid", "0"),
            ],
            blocks: vec![],
        });
        vmf.inner.blocks.push(Block {
            name: "world".into(),
            props: vec![
                Property::new("mapversion", "1"),
                Property::new("classname", "worldspawn"),
                Property::new("skyname", self.options.sky_name),
            ],
            blocks: solid_blocks,
        });
        // ENTS HERE
        vmf.inner.blocks.extend(self.entities.iter().map(|e| e.to_lower()));
        // cameras unnessesary
        // cordons unnessesary

        vmf
    }
}

impl<'a> ToLower<Block<StrType<'a>>> for Solid<'a> {
    fn into_lower(self) -> Block<StrType<'a>> {
        Block {
            name: "solid".into(),
            // id unnecessary as [`vmf_parser_nom`] can generate new ids
            props: vec![],
            blocks: self.sides.iter().map(|x| x.to_lower()).collect(),
        }
    }
}

impl<'a> ToLower<Block<StrType<'a>>> for Side<'a> {
    fn into_lower(self) -> Block<StrType<'a>> {
        let props = vec![
            // id unnecessary as [`vmf_parser_nom`] can generate new ids
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
        let blocks = match self.disp {
            Some(disp) => {
                let disp_info = disp.into_disp_info();
                vec![disp_info.into_lower()]
            }
            None => vec![],
        };
        Block::new("side", props, blocks)
    }
}

// impl ToLower<DispInfo> for Displacement {
//     fn to_lower(&self) -> DispInfo {
//         self.clone().into_disp_info()
//     }
// }

impl<'a> ToLower<Block<StrType<'a>>> for DispInfo {
    fn into_lower(self) -> Block<StrType<'a>> {
        use std::fmt::Write;
        let mut props = vec![
            Property::new("power", self.power.to_string()),
            Property::new("startposition", format!("[{}]", self.start_position)),
            Property::new("flags", self.flags.to_string()),
            Property::new("elevation", self.elevation.to_string()),
            Property::new("subdiv", (self.is_subdiv as i32).to_string()),
        ];
        let mut allowed_verts = String::new();
        for int in self.allowed_verts {
            write!(&mut allowed_verts, "{} ", int).unwrap();
        }
        allowed_verts.truncate(allowed_verts.len() - 1);
        let allowed_verts = Property::new("10", allowed_verts);
        let blocks = vec![
            Block::new("normals", self.normals.to_lower(), vec![]),
            Block::new("distances", self.distances.to_lower(), vec![]),
            Block::new("offsets", self.offsets.to_lower(), vec![]),
            Block::new("offset_normals", self.offset_normals.to_lower(), vec![]),
            Block::new("alphas", self.alphas.to_lower(), vec![]),
            Block::new("triangle_tags", self.triangle_tags.to_lower(), vec![]),
            Block::new("allowed_verts", vec![allowed_verts], vec![]),
        ];
        Block::new("dispinfo", props, blocks)
    }
}

impl<'a> ToLower<Block<StrType<'a>>> for Entity<StrType<'a>> {
    fn into_lower(self) -> Block<StrType<'a>> {
        Block {
            // name: self.classname.into(),
            name: "entity".into(),
            props: self.props,
            blocks: vec![],
        }
    }
}

impl<'a, T: Clone + Display> ToLower<Vec<Property<StrType<'a>, StrType<'a>>>> for Vec2d<T> {
    /// https://developer.valvesoftware.com/wiki/.vmf#Normals
    fn to_lower(&self) -> Vec<Property<StrType<'a>, StrType<'a>>> {
        use std::fmt::Write;
        let mut i = 0;
        let row_to_prop = |row: &[T]| {
            let key = format!("row{}", i);
            i += 1;

            // rigmarole for join(" ")
            let mut value = String::new();

            // let mut row = row.to_vec();
            // row.reverse();
            for item in row {
                write!(&mut value, "{item} ").unwrap();
            }
            value.truncate(value.len() - 1);

            Property::new(key, value)
        };

        // let mut rows = self.rows().collect::<Vec<_>>();
        // rows.reverse(); // FIXME:HACK:
        let rows = self.rows();

        rows.into_iter().map(row_to_prop).collect()
    }
}

// TODO: better. rn only for tests
// TODO: FIXME: NOW: git and move and parse stuff instead of floating
pub(crate) fn block_to_solid<'a>(block: &'a Block<impl AsRef<str>>) -> Solid<'a> {
    assert_eq!(block.name.as_ref(), "solid");

    // just ignore props lol
    // Blocks
    let mut sides = Vec::with_capacity(6); // usually 6
    for block in block.blocks.iter() {
        match block.name.as_ref() {
            "side" => {
                sides.push(parse_side(block));
            }
            "editor" => {}
            block => panic!("Unexpected block {block}"),
        }
    }

    Solid::new(sides)
}

pub(crate) fn parse_side<'a>(block: &'a Block<impl AsRef<str>>) -> Side<'a> {
    assert_eq!(block.name.as_ref(), "side");

    // Props
    let mut plane = None;
    let mut material = None;
    let mut uaxis = None;
    let mut vaxis = None;
    let mut lightmapscale = None;
    let mut smoothing_groups = None;
    for prop in block.props.iter() {
        let Property { key, value } = prop;
        let key = key.as_ref();
        let value = value.as_ref();

        match key {
            "id" => {}
            "plane" => {
                // "plane" "(128 128 128) (128 -128 128) (-128 -128 128)"
                let s = value.trim_start_matches('(').trim_end_matches(')');
                let spaces_split = s.split(") (");
                // NOTE: chunks/next_chunk would be good when stabilized
                // or like actual good parsing
                let triplets: Vec<Vector3<f32>> = spaces_split
                    .map(|three_floats| {
                        let three_floats: Vec<f32> = three_floats
                            .split(' ')
                            .map(|float| float.parse::<f32>().expect("Error parsing  float"))
                            .collect();
                        assert_eq!(three_floats.len(), 3, "A point must have 3 numbers");
                        let array: [_; 3] = three_floats.try_into().unwrap();
                        Vector3::from(array)
                    })
                    .collect();

                // NOTE: most useless required type hint
                let [bl, tl, tr]: [_; 3] = triplets.try_into().expect("A plane must have 3 points");

                plane = Some(Plane::new(bl, tl, tr));
            }
            "material" => {
                material = Some(value);
            }
            "uaxis" => {
                uaxis = Some(parse_uvaxis(value));
            }
            "vaxis" => {
                vaxis = Some(parse_uvaxis(value));
            }
            "rotation" => {}
            "lightmapscale" => {
                lightmapscale = value.parse::<u8>().ok();
            }
            "smoothing_groups" => {
                smoothing_groups = value.parse::<i32>().ok();
            }
            key => panic!("Unexpected key {key}"),
        }
    }

    // Blocks
    let mut disp_info = None;
    for block in block.blocks.iter() {
        let name = block.name.as_ref();
        match name {
            "dispinfo" => {
                disp_info = Some(parse_dispinfo(block).unwrap());
            }
            block => panic!("Unexpected block {block}"),
        }
    }

    let texture = Texture::new(
        material.unwrap().into(),
        uaxis.unwrap(),
        vaxis.unwrap(),
        lightmapscale.unwrap(),
    );
    let plane = plane.unwrap();
    let disp = disp_info.map(|disp_info| Displacement::from_disp_info(disp_info, plane.clone()));

    Side { plane, texture, disp }
}

fn parse_uvaxis(s: &str) -> UVAxis<f32> {
    // iter over just floats
    let mut floats = s
        .split(|char| "[] ".contains(char))
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<f32>().expect("Error parsing f32"));
    let [x, y, z, trans, scale] = floats.next_chunk2().expect("Bad UVAxis");
    assert!(floats.next().is_none(), "Too many");
    UVAxis::new(x, y, z, trans, scale)
}

fn parse_dispinfo<'a>(block: &Block<impl AsRef<str>>) -> Result<DispInfo, &'static str> {
    assert_eq!(block.name.as_ref(), "dispinfo");

    // Props
    let mut power = None;
    let mut start_position = None;
    let mut flags = None;
    let mut elevation = None;
    let mut is_subdiv = None;

    for prop in block.props.iter() {
        let Property { key, value } = prop;
        let key = key.as_ref();
        let value = value.as_ref();

        match key {
            "power" => {
                power = value.parse::<u32>().ok();
            }
            "startposition" => {
                start_position = value.parse::<Vector3<_>>().ok();
            }
            "flags" => {
                flags = value.parse::<i32>().ok();
            }
            "elevation" => {
                elevation = value.parse::<f32>().ok();
            }
            "subdiv" => {
                is_subdiv = value.parse::<i32>().ok();
            }
            _ => return Err("Unexpected key"),
        }
    }

    // Blocks
    let mut normals = None;
    let mut distances = None;
    let mut offsets = None;
    let mut offset_normals = None;
    let mut alphas = None;
    let mut triangle_tags = None;
    let mut allowed_verts = None;

    for block in block.blocks.iter() {
        let name = block.name.as_ref();

        match name {
            "normals" => {
                normals = parse_vec2d::<f32>(block).ok();
            }
            "distances" => {
                distances = parse_vec2d::<f32>(block).ok();
            }
            "offsets" => {
                offsets = parse_vec2d::<f32>(block).ok();
            }
            "offset_normals" => {
                offset_normals = parse_vec2d::<f32>(block).ok();
            }
            "alphas" => {
                alphas = parse_vec2d::<f32>(block).ok();
            }
            "triangle_tags" => {
                triangle_tags = parse_vec2d::<i32>(block).ok();
            }
            "allowed_verts" => {
                allowed_verts = parse_seps_chunks_exact(block.props[0].value.as_ref(), " ").ok();
            }
            _ => return Err("Unexpected block"),
        }
    }

    let normals = vec2d_f32_to_vector3(normals.ok_or("Normal misparse or missing")?);
    let offsets = vec2d_f32_to_vector3(offsets.ok_or("offsets misparse or missing")?);
    let offset_normals =
        vec2d_f32_to_vector3(offset_normals.ok_or("offset_normals misparse or missing")?);

    // let normals = {normals.f32s_to_vector3(normals.unwrap().inner);}
    // TODO: allow default values for resilience? ex: default flags
    Ok(DispInfo {
        power: power.ok_or("power is missing")?,
        start_position: start_position.ok_or("start_position is missing")?,
        flags: flags.ok_or("flags is missing")?,
        elevation: elevation.ok_or("elevation is missing")?,
        is_subdiv: is_subdiv.ok_or("is_subdiv is missing")? != 0,
        normals,
        distances: distances.ok_or("distances is missing")?,
        offsets,
        offset_normals,
        alphas: alphas.ok_or("alphas is missing")?,
        triangle_tags: triangle_tags.ok_or("triangle_tags is missing")?,
        allowed_verts: allowed_verts.ok_or("allowed_verts is missing")?,
    })
}

/// Helper function.
/// `split()` on any `seperators` ignore empty strings `parse()` returning array of length `N`
/// return first error if any.
fn parse_seps_chunks_exact<const N: usize, T: FromStr>(
    s: &str, seperators: &str,
) -> Result<[T; N], &'static str> {
    // strs between seperators ignoring empty
    let mut strs = s.split(|char| seperators.contains(char)).filter(|s| !s.is_empty());
    let str_arr = strs.next_chunk2().ok_or("Too short")?;
    if strs.next().is_some() {
        // too many
        return Err("Too long");
    }
    let parsed = str_arr.try_map2(|s| s.parse::<T>()).map_err(|_| "Error parsing")?;

    Ok(parsed)
}

/// Helper function.
/// `split()` on any `seperators` ignore empty strings `parse()` returning a vec
/// return first error if any.
fn parse_seps_chunks<'a, 'b, T: FromStr + std::fmt::Debug>(
    s: &'a str, seperators: &'b str,
) -> Result<Vec<T>, &'static str>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    // strs between seperators ignoring empty
    let strs = s.split(|char| seperators.contains(char)).filter(|s| !s.is_empty());
    dbg!(strs.clone().map(|s| s.parse::<T>()).collect::<Vec<_>>());
    strs.map(|s| s.parse::<T>().map_err(|_| "Error parsing")).collect()
}

impl FromStr for Vector3<f32> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        dbg!(s);
        match parse_seps_chunks_exact(s, "()[] ") {
            Ok(array) => Ok(Vector3::from(array)),
            Err(_) => Err("Error parsing Vector3"),
        }
    }
}

fn vec2d_f32_to_vector3(vec2d: Vec2d<f32>) -> Vec2d<Vector3<f32>> {
    Vec2d::from_parts(f32s_to_vector3s(&vec2d.inner), [vec2d.strides[0] / 3, 1])
}

fn f32s_to_vector3s(f32s: &[f32]) -> Vec<Vector3<f32>> {
    f32s.chunks_exact(3)
        .map(|slice| {
            let arr: [_; 3] = slice.try_into().unwrap();
            Vector3::from(arr)
        })
        .collect()
}

fn parse_vec2d<'a, T: FromStr + std::fmt::Debug>(
    block: &Block<impl AsRef<str>>,
) -> Result<Vec2d<T>, &'static str>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    let mut output = Vec::new();
    // TODO: better way to ensure all rows are the same length. hashmap or mut option
    let mut lens = Vec::with_capacity(block.props.len());

    for (i, prop) in block.props.iter().enumerate() {
        let Property { key, value } = prop;
        let key = key.as_ref();
        let value = value.as_ref();
        let row_key = format!("row{i}");

        // let mut vec2d = Vec2d::new(Vec2d::strides(1));
        if row_key == key {
            let values = parse_seps_chunks(value, " ")?.into_iter();
            lens.push(values.len());
            output.extend(values);
        } else {
            return Err("Bad row key");
        }
    }

    let width = lens.first().copied().unwrap_or(0);
    if !lens.iter().copied().all(|v| v == width) {
        return Err("Rows must be the same length");
    }

    Ok(Vec2d::from_parts(output, Vec2d::strides(width)))
}

// /// Trait to convert into a low level map element representation.
// /// Example: A mid level [`Solid`](crate::map::solid) into a low level [`Block`](vmf_parser_nom::ast::Block)
// pub trait ToLowLevel {
//     type Target: ?Sized;
//     /// Convert this into a low level map element, consuming `self`.
//     fn into_low_level(self) -> Self::Target;
//     /// Convert this into an owned low level map element.
//     fn to_low_level(&self) -> Self::Target;
// }

// /// Trait to convert into a mid level map element representation.
// /// Example: A high level <TODO: ROOM> into a mid level vec [`Solid`](crate::map::solid)s
// pub trait ToMidLevel {
//     type Target: ?Sized;
//     /// Convert this into a mid level map element, consuming `self`.
//     fn into_low_level(self) -> Self::Target;
//     /// Convert this into an owned mid level map element.
//     fn to_low_level(&self) -> Self::Target;
// }

// /// Trait to convert into a high level map element representation.
// /// Example: A vec of solids into a room.
// pub trait ToHighLevel {
//     type Target: ?Sized;
//     /// Convert this into a high level map element, consuming `self`.
//     fn into_low_level(self) -> Self::Target;
//     /// Convert this into an owned high level map element.
//     fn to_low_level(&self) -> Self::Target;
// }

// use std::{ops::{Deref, DerefMut}, fmt::Display};

// pub use vmf_impl::*;
// use vmf_parser_nom::ast::{Block, Property};

// #[derive(Clone, Debug, Default)]
// pub struct Vmf<S>(Vec<Block<S>>);

// pub trait ToVmf<S, T, E> {
//     /// Convert into vmf ast.
//     fn to_vmf(&self, state: &mut T) -> Vmf<S>;
// }

// pub trait ToBlock<S, T, E> {
//     /// Convert into vmf [`Block`].
//     fn to_block(&self, state: &mut T) -> Block<S>;
// }

// pub trait ToProps<T, K, V, E> {
//     /// Convert into vmf [`Property`]s.
//     fn to_props(&self, state: &mut T) -> Vec<Property<K, V>>;
// }

// pub trait FromVmf<T, U, E>
// where
//     Self: Sized,
// {
//     /// Parse from a part of vmf file.
//     fn from_vmf(vmf: Vmf<U>, state: &mut T) -> Result<Self, E>;
// }

// // convenience traits:

// // pub trait PushProp<T, K> {
// //     fn push_prop(&mut self, key: T, value: String);
// // }

// // impl<T, K> PushProp<T, K> for Vec<Property<K>>
// // where
// //     T: Into<K>,
// // {
// //     fn push_prop(&mut self, key: T, value: String) {
// //         self.push(Property { key: key.into(), value })
// //     }
// // }

// pub trait PropertyExt<T, K> {
//     fn new(key: T, value: String) -> Self;
// }

// impl<S> Deref for Vmf<S> {
//     type Target = Vec<Block<S>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<S> DerefMut for Vmf<S> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Display> Display for Vmf<S> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "// auto generated vmf file")?;
//         for block in self.iter() {
//             write!(f, "{block:#}")?;
//         }
//         Ok(())
//     }
// }

// // impl<T, K, V> PropertyExt<T, K> for Property<K, V>
// // where
// //     T: Into<K>,
// // {
// //     fn new(key: T, value: String) -> Property<K, V> {
// //         Property { key: key.into(), value }
// //     }
// // }
