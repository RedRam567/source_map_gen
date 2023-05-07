#![allow(clippy::print_literal)] // false positive for file!()

use std::iter::Peekable;

use crate::prelude::{Material, Plane, Side, Solid, Texture, Vector2, Vector3};
use crate::IterWithNext;

// cube      6 textures, aligned, bounds
// wedges    5 textures, aligned, bounds, rotation
// spikes    2 textures, aligned, bounds, num, rotation2
// cylinders 3 textures, aligned, bounds, num, rotation2
// rounded wedges? 5 textures, aligned, bounds, num, rotation2
// arches    6 textures, aligned, bounds, num, angle, z
// spheres   1 textures, aligned, bounds, numxy, numz, rotation2

// cube     top bottom north sound east west
// wedge    top, bottom, north, sound, east
// cone     bottom, side
// cylinder top, bottom, side
// slice?   top, bottom, south, east, nw side
// sphere   side
// arch     .

// yeah top looks better -> eh top

// cube
// wedge    east north sound top bottom
// cone     side bottom
// cylinder side top bottom
// slice?   side south east top bottom
// sphere   side
// arch

// solid transform
// hmm all, bounds, stretch sphere hmm

// generate vertexes, make planes, wrap in solid
// TRY TO ALLIGN TEXTURES
// world/face allign, abs/relative
// point/face on axes circles

// make struct ur just methods strait to solids?
// uh lets not go down oop hell and just do solids

/// Bounds in 3d space.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Bounds {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Bounds {
    /// Create bounds from any two points in 3d space.
    pub fn new(point1: Vector3<f32>, point2: Vector3<f32>) -> Self {
        Self {
            min: Vector3 {
                x: point1.x.min(point2.x),
                y: point1.y.min(point2.y),
                z: point1.z.min(point2.z),
            },
            max: Vector3 {
                x: point1.x.max(point2.x),
                y: point1.y.max(point2.y),
                z: point1.z.max(point2.z),
            },
        }
    }

    /// Returns the 8 vertexes of the [`Bounds`] in the order:
    ///
    /// `west south bottom`, `west north bottom`, `east north bottom`, `east south bottom`
    ///
    /// `west south top`, `west north top`, `east north top`, `east south top`
    pub const fn verts(&self) -> [Vector3<f32>; 8] {
        // TODO: change spiral order, way too lazy
        [
            Vector3 { ..self.min },                               // 0 south west bottom
            Vector3 { y: self.max.y, ..self.min },                // 1 north west bottom
            Vector3 { x: self.max.x, y: self.max.y, ..self.min }, // 2 north east bottom
            Vector3 { x: self.max.x, ..self.min },                // 3 south east bottom
            Vector3 { x: self.min.x, y: self.min.y, ..self.max }, // 4 south west top
            Vector3 { x: self.min.x, ..self.max },                // 5 north west top
            Vector3 { ..self.max },                               // 6 north east top
            Vector3 { y: self.min.y, ..self.max },                // 7 south east top
        ]
    }

    /// The center of `self` on the XY plane.
    pub fn center_xy(&self) -> Vector2<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let y = (self.min.y + self.max.y) / 2.0;
        Vector2::new(x, y)
    }

    /// The center of `self` on the XY plane.
    pub fn center_yz(&self) -> Vector2<f32> {
        let y = (self.min.y + self.max.y) / 2.0;
        let z = (self.min.z + self.max.z) / 2.0;
        Vector2::new(y, z)
    }

    /// The center of `self` on the XY plane.
    pub fn center_xz(&self) -> Vector2<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let z = (self.min.z + self.max.z) / 2.0;
        Vector2::new(x, z)
    }

    pub fn center(&self) -> Vector3<f32> {
        let x = (self.min.x + self.max.x) / 2.0;
        let y = (self.min.y + self.max.y) / 2.0;
        let z = (self.min.z + self.max.z) / 2.0;
        Vector3::new(x, y, z)
    }

    pub fn x_len(&self) -> f32 {
        (self.min.x - self.max.x).abs()
    }

    pub fn y_len(&self) -> f32 {
        (self.min.y - self.max.y).abs()
    }

    pub fn z_len(&self) -> f32 {
        (self.min.z - self.max.z).abs()
    }

    pub const fn top_plane(&self) -> Plane {
        // 7 6 5
        Plane::new(
            Vector3 { x: self.min.x, ..self.max },
            Vector3 { ..self.max },
            Vector3 { y: self.min.y, ..self.max },
        )
    }

    pub const fn bottom_plane(&self) -> Plane {
        // 2, 1, 0
        Plane::new(
            Vector3 { x: self.max.x, y: self.max.y, ..self.min },
            Vector3 { y: self.max.y, ..self.min },
            Vector3 { ..self.min },
        )
    }

    pub fn translate(mut self, trans: &Vector3<f32>) -> Self {
        self.translate_mut(trans);
        self
    }

    pub fn translate_mut(&mut self, trans: &Vector3<f32>) -> &mut Self {
        self.min += trans;
        self.max += trans;
        self
    }
}

/// Returns an iterator to points on an ellipse starting north (+Y) and heading
/// east (+X, right, closewise) which is the same order Hammer seems to use.
/// All math is done internally as `f64` as an (in)sanity check.
///
/// See also <https://en.wikipedia.org/wiki/Ellipse>
pub fn ellipse_verts(
    center: Vector2<f32>, x_radius: f32, y_radius: f32, mut num_sides: u32, options: &SolidOptions,
) -> impl ExactSizeIterator<Item = Vector2<f32>> + Clone {
    assert!(x_radius > 0.0 && y_radius > 0.0);
    if num_sides < 3 {
        // try not to panic
        eprintln!("Warning Ellipse: num_sides must be > 3, clamped");
        num_sides = 3;
    }
    // clamp for too small for sides
    if !options.allow_frac {
        // max sides before int rounding give 3 colinear points (2 coplanar planes)
        let max_sides = (x_radius).min(y_radius) as u32 / 2;
        if num_sides > max_sides {
            eprintln!("[{}:{}] Warning Ellipse: Sides clamped. Colinear ellipse points, too small/too many sides. Ellipse(x:{:.1},y:{:.1},x{})1",
                file!(), line!(), x_radius, y_radius, num_sides);
            num_sides = max_sides;
        }
        num_sides = num_sides.min(max_sides);
    }

    // relative to north, right/clockwise(east)
    // RangeInclusive<u32> doesnt impl ExactSizeIterator BUT Range<u32> does
    // BUT neither range impl for 64/128 ints WTF???
    // https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html#implementors
    let delta_angle = std::f64::consts::TAU / num_sides as f64;
    (0..num_sides).map(move |n| {
        let angle = delta_angle * (n + 1) as f64;
        let x = x_radius as f64 * -angle.cos() + center.x as f64;
        let y = y_radius as f64 * angle.sin() + center.y as f64;
        Vector2::new(x as f32, y as f32)
    })
}

pub const SWB: usize = 0;
pub const NWB: usize = 1;
pub const NEB: usize = 2;
pub const SEB: usize = 3;
pub const SWT: usize = 4;
pub const NWT: usize = 5;
pub const NET: usize = 6;
pub const SET: usize = 7;
// let top = Plane::new(verts[4].const_clone(), verts[5].const_clone(), verts[6].const_clone()).with_texture(textures[0]);
// let bottom = Plane::new(verts[2].const_clone(), verts[1].const_clone(), verts[0].const_clone()).with_texture(textures[1]);
// let north = Plane::new(verts[2].const_clone(), verts[6].const_clone(), verts[5].const_clone()).with_texture(textures[2]);
// let south = Plane::new(verts[0].const_clone(), verts[4].const_clone(), verts[7].const_clone()).with_texture(textures[3]);
// let east = Plane::new(verts[3].const_clone(), verts[7].const_clone(), verts[6].const_clone()).with_texture(textures[5]);
// let west = Plane::new(verts[1].const_clone(), verts[5].const_clone(), verts[4].const_clone()).with_texture(textures[4]);

// upgrade to frac if too small?
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SolidOptions {
    pub allow_frac: bool,
    pub world_align: bool,
}

// TODO: sphere, arch, multi

// TODO:DOCS:
/// top, bottom, north, south, east, west
#[rustfmt::skip]
fn cube<'a>(bounds: &Bounds, options: &SolidOptions, materials: &[&Material<'a>; 6]) -> Solid<'a> {
    let verts = bounds.verts();

    // let top = Side::new_parts_option(verts[SWT].clone(), verts[NWT].clone(), verts[NET].clone(), materials[0], options);
    // let top = Plane::new(verts[SWT].const_clone(), verts[NWT].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[0], align);
    // let bottom = Plane::new(verts[NEB].const_clone(), verts[NWB].const_clone(), verts[SWB].const_clone()).with_material_alignment(materials[1], align);
    // let north = Plane::new(verts[NEB].const_clone(), verts[NET].const_clone(), verts[NWT].const_clone()).with_material_alignment(materials[2], align);
    // let south = Plane::new(verts[SWB].const_clone(), verts[SWT].const_clone(), verts[SET].const_clone()).with_material_alignment(materials[3], align);
    // let east = Plane::new(verts[SEB].const_clone(), verts[SET].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[4], align);
    // let west = Plane::new(verts[NWB].const_clone(), verts[NWT].const_clone(), verts[SWT].const_clone()).with_material_alignment(materials[5], align);
    let top = Side::new_points(verts[SWT].clone(), verts[NWT].clone(), verts[NET].clone(), materials[0], options);
    let bottom = Side::new_points(verts[NEB].clone(), verts[NWB].clone(), verts[SWB].clone(), materials[1], options);
    let north = Side::new_points(verts[NEB].clone(), verts[NET].clone(), verts[NWT].clone(), materials[2], options);
    let south = Side::new_points(verts[SWB].clone(), verts[SWT].clone(), verts[SET].clone(), materials[3], options);
    let east = Side::new_points(verts[SEB].clone(), verts[SET].clone(), verts[NET].clone(), materials[4], options);
    let west = Side::new_points(verts[NWB].clone(), verts[NWT].clone(), verts[SWT].clone(), materials[5], options);

    Solid::new(vec![top, bottom, north, south, east, west])
}

// TODO:DOCS:
/// west/top slope, bottom, north, south, east
#[rustfmt::skip]
fn wedge<'a>(bounds: &Bounds, materials: &[&Material<'a>; 5], options: &SolidOptions) -> Solid<'a> {
    let verts = bounds.verts();

    // same as top but with first two verts on bottom
    // let slope = Plane::new(verts[SWB].const_clone(), verts[NWB].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[0], align);
    // let bottom = Plane::new(verts[NEB].const_clone(), verts[NWB].const_clone(), verts[SWB].const_clone()).with_material_alignment(materials[1], align);
    // let north = Plane::new(verts[NEB].const_clone(), verts[NET].const_clone(), verts[NWT].const_clone()).with_material_alignment(materials[2], align);
    // let south = Plane::new(verts[SWB].const_clone(), verts[SWT].const_clone(), verts[SET].const_clone()).with_material_alignment(materials[3], align);
    // let east = Plane::new(verts[SEB].const_clone(), verts[SET].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[4], align);
    let slope = Side::new_points(verts[SWB].clone(), verts[NWB].clone(), verts[NET].clone(), materials[0], options);
    let bottom = Side::new_points(verts[NEB].clone(), verts[NWB].clone(), verts[SWB].clone(), materials[1], options);
    let north = Side::new_points(verts[NEB].clone(), verts[NET].clone(), verts[NWT].clone(), materials[2], options);
    let south = Side::new_points(verts[SWB].clone(), verts[SWT].clone(), verts[SET].clone(), materials[3], options);
    let east = Side::new_points(verts[SEB].clone(), verts[SET].clone(), verts[NET].clone(), materials[4], options);

    Solid::new(vec![slope, bottom, north, south, east])
}

// top, down, "right"
// TODO: wibbly cone? arbitaray tip
/// bottom, sides
/// Uses same points as Hammer for the sides but different for the base.
/// # Note
/// Hammer (l4d2) seems to have a lot of problems with non-powers of two `num_sides` and
/// `bounds` that are roughly as tall as wide. Unless you know what you are doing, I would recommend
/// 16 sides max. Hammer has absolute limit of ~128 total faces
///
/// `vbsp` is much more forgiving and seems to allow all spikes with an absolute
/// max of 63 sides (64 total faces), contrary to the [Valve Wiki] which says 128.
///
/// [Valve Wiki]: https://developer.valvesoftware.com/wiki/Brush
fn spike<'a>(
    bounds: &Bounds, num_sides: u32, mats: &[&Material<'a>; 2], options: &SolidOptions,
) -> Solid<'a> {
    // generate circle
    // let bottom_center = bounds.center_xy().with_z(bounds.min.z);
    // let top_center = bounds.center_xy().with_z(bounds.max.z);
    let bottom = bounds.min.z;
    let top = bounds.max.z;
    let center_xy = bounds.center_xy();
    let top_point = center_xy.with_z(top);

    // dbg!(&top_center, &bottom_center);

    // make base
    let mut sides = Vec::with_capacity(num_sides as usize + 1);
    sides.push(bounds.bottom_plane().with_mat_align(mats[0], options.world_align));

    // get iter to point on circle and next
    let circle_verts = IterWithNext::new(ellipse_verts(
        center_xy,
        bounds.x_len() / 2.0,
        bounds.y_len() / 2.0,
        num_sides,
        options,
    ));

    // make spike sides
    for (current, next) in circle_verts {
        let current = current.with_z(bottom);
        let next = next.with_z(bottom);
        sides.push(Side::new_points(top_point.clone(), current, next, mats[1], options));
        // sides.push(Side::new_points(next, top_point.clone(), current, materials[1], options));
        // sides.push(Side::new_points(current, next, top_point.clone(), materials[1], options));
    }

    Solid::new(sides)
}

/// top, bottom, sides
fn cylinder<'a>(
    bounds: &Bounds, num_sides: u32, mats: [&Material<'a>; 3], options: &SolidOptions,
) -> Solid<'a> {
    let bottom = bounds.min.z;
    let top = bounds.max.z;
    let center_xy = bounds.center_xy();

    // make bases
    let mut sides = Vec::with_capacity(num_sides as usize + 2);
    sides.push(bounds.top_plane().with_mat_align(mats[0], options.world_align));
    sides.push(bounds.bottom_plane().with_mat_align(mats[1], options.world_align));

    // get iter to point on circle and next
    let circle_verts = IterWithNext::new(ellipse_verts(
        center_xy,
        bounds.x_len() / 2.0,
        bounds.y_len() / 2.0,
        num_sides,
        options,
    ));

    // make cylinder sides
    for (current, next) in circle_verts {
        let top_point = next.clone().with_z(top);
        let current = current.with_z(bottom);
        let next = next.with_z(bottom);
        dbg!(&top_point, &current, &next);
        sides.push(Side::new_points(current, next, top_point, mats[2], options));
    }

    Solid::new(sides)
}

/// Top, Bottom, sides
/// A [`cylinder`] with different sized (and offset) bases. Bases must have the same number of sides
fn frustum<'a, I>(
    top: I, top_z: f32, bottom: I, bottom_z: f32, mats: &[&Material<'a>; 3], options: &SolidOptions,
) -> Solid<'a>
where
    I: ExactSizeIterator<Item = Vector2<f32>>,
{
    assert_eq!(bottom.len(), top.len());
    assert!(top.len() >= 3);

    let mut sides = Vec::with_capacity(top.len() + 2);
    // sides.push(bounds.top_plane().with_mat_align(mats[0], options.world_align));
    sides.push(Plane::top(top_z).with_mat_align(mats[0], options.world_align));
    sides.push(Plane::bottom(bottom_z).with_mat_align(mats[1], options.world_align));

    let bottom_verts = IterWithNext::new(bottom);
    for ((current, next), top) in bottom_verts.zip(top) {
        sides.push(Side::new_points(
            current.with_z(bottom_z),
            next.with_z(bottom_z),
            top.with_z(top_z),
            mats[2],
            options,
        ));
    }

    Solid::new(sides)
}

// get heights
// get "circumference"/"radius" of sphere at height
// bunch of frustums
fn sphere<'a>(
    bounds: &Bounds, num_sides: u32, mats: &[&Material<'a>; 1], options: &SolidOptions,
) -> Vec<Solid<'a>> {
    const SPHERE_INNER: Material = crate::map::texture::NO_DRAW;
    let frustum_mats = &[&SPHERE_INNER, &SPHERE_INNER, mats[0]];
    let spike_mats = &[&SPHERE_INNER, mats[0]];

    dbg!(bounds);

    // let bottom = bounds.min.z;
    // let top = bounds.max.z;
    let center_yz = bounds.center_yz();
    let center_xy = bounds.center_xy();
    let center = dbg!(bounds.center());
    let center_z = bounds.center().z;
    let radius_x = bounds.x_len() / 2.0;
    let radius_y = bounds.y_len() / 2.0;

    // // verts on the vertical yz plane (from +X, looking at origin)
    // // TODO: order, TODO: point
    // let verts_yz = IterWithNext::new(ellipse_verts(
    //     center_yz,
    //     dbg!( bounds.y_len() / 2.0),
    //     dbg!( bounds.z_len() / 2.0),
    //     num_sides,
    //     options,
    // ));
    // let verts_xyz = verts_yz.map(|(a, b)| (Vector3::new(center.x, a.x, a.y), Vector3::new(center.x, b.x, b.y)));
    // // dbg!(verts_xyz.clone().map(|v| v.0).collect::<Vec<_>>());

    // let verts_xyz = verts_xyz.collect::<Vec<_>>();
    // let mut verts_xyz = verts_xyz.into_iter();

    // let _top_point = verts_xyz.next();
    // let _bottom_point = verts_xyz.next_back();
    // let mut sides = Vec::new();
    // for (current, next) in verts_xyz {
    //     // frustum(bottom, bottom_z, top, top_z, mats, options)
    //     // frustum(top, top_z, bottom, bottom_z, mats, options)
    //     let top_radius_x = center.into_vector2().dist(&current.into_vector2());
    //     let top_base = IterWithNext::new(ellipse_verts(current.into_vector2(), x_radius, y_radius, num_sides, options))
    //     // frustum()
    // }

    // split sphere into equal height sections from the top
    // TODO: use angles instead or trig
    let mut heights = (1..num_sides).map(|n| {
        // 0.5 -> -0.5 as n -> num_sides
        let multiplier = (num_sides - n) as f32 / num_sides as f32 - 0.5;
        let height = center_z + multiplier * bounds.z_len();
        if options.allow_frac {
            height
        } else {
            height.round()
        }
    });

    let top_layer = heights.clone().next().unwrap() - center_z;
    let bottom_layer = heights.clone().next_back().unwrap() - center_z;
    // dbg!(heights.clone().collect::<Vec<_>>());

    let heights = IterWithNext::new(heights);
    let mut solids = Vec::new();
    // make tops and bottoms
    // sides.push(bounds.top_plane().with_mat_align(mats[1], options.world_align)); // TODO: proper
    // sides.push(bounds.bottom_plane().with_mat_align(mats[1], options.world_align)); // TODO: proper
    let top_xy = sphere_circle_verts_at_height(radius_x, radius_y, top_layer, options.allow_frac);
    let bottom_xy = sphere_circle_verts_at_height(radius_x, radius_y, bottom_layer, options.allow_frac);

    // NOTE: height absolute
    let top_bounds = Bounds::new(top_xy.with_z(top_layer + center_z), (-top_xy).with_z(bounds.max.z));
    let bottom_bounds =
        Bounds::new(bottom_xy.with_z(bottom_layer + center_z), (-bottom_xy).with_z(bounds.min.z));

    dbg!(&top_bounds, &bottom_bounds);
    solids.push(spike(&top_bounds, num_sides, spike_mats, options));
    solids.push(spike(&bottom_bounds, num_sides, spike_mats, options)); // TODO: transform
    // let x = spike(bounds, num_sides, mats, options)
    for (height_top, height_bottom) in heights {
        let height_top_from_center = height_top - center_z;
        let height_bottom_from_center = height_bottom - center_z;
        dbg!(height_top, height_bottom);
        dbg!(height_top_from_center, height_bottom_from_center);

        // get radii
        let top_radius_x = sphere_circle_radius_at_height(radius_x, height_top_from_center, options.allow_frac);
        let top_radius_y = sphere_circle_radius_at_height(radius_y, height_top_from_center, options.allow_frac);
        let bottom_radius_x = sphere_circle_radius_at_height(radius_x, height_bottom_from_center, options.allow_frac);
        let bottom_radius_y = sphere_circle_radius_at_height(radius_y, height_bottom_from_center, options.allow_frac);

        // make base circle
        let top_circle =
        ellipse_verts(center_xy.clone(), top_radius_x, top_radius_y, num_sides, options);
        let bottom_circle =
        ellipse_verts(center_xy.clone(), bottom_radius_x, bottom_radius_y, num_sides, options);
        // make frustum layer
        // NOTE: height absolute
        let frustum =
            frustum(top_circle, height_top, bottom_circle, height_bottom, frustum_mats, options);
        solids.push(frustum);
    }
    solids
}

/// See <https://en.wikipedia.org/wiki/Circle_of_a_sphere>
fn sphere_circle_radius_at_height(radius: f32, height_from_center: f32, allow_frac: bool) -> f32 {
    let height = height_from_center;
    dbg!(f32::sqrt(radius * radius - height * height))
}

fn sphere_circle_verts_at_height(
    x_radius: f32, y_radius: f32, height_from_center: f32, allow_frac: bool
) -> Vector2<f32> {
    let x = sphere_circle_radius_at_height(x_radius, height_from_center, allow_frac);
    let y = sphere_circle_radius_at_height(y_radius, height_from_center, allow_frac);
    Vector2::new(x, y)
}

#[cfg(test)]
mod tests {
    use vmf_parser_nom::ast::Vmf;

    use crate::map::Map;
    use crate::vmf::ToLower;
    use crate::StrType;

    use super::*;

    // #[test]
    // fn circle() {
    //     // let truth = [];
    //     let result = ellipse_verts(Vector3::default(), 64.0, 16);
    //     dbg!(result.collect::<Vec<_>>());
    //     panic!();
    //     // for i in truth.iter().zip(result) {
    //     //     // assert!(tu)
    //     // }
    // }

    #[ignore]
    #[test]
    fn spike_test() {
        let mut map = Map::default();
        let options = SolidOptions::default();
        // let options = SolidOptions { allow_frac: false, ..Default::default() };
        // let options = SolidOptions { allow_frac: false, world_align: false };
        // let options = SolidOptions { allow_frac: true, ..Default::default() };

        // TODO: why cant I no longer do 127? or even 64?
        // WTF NOW I CANT EVEN USE 32 FROM HAMMER!!
        // I THINK 64 is vbsps limit (wiki says 128), I THINK 32 is hammers limit (fucking rip) -> split
        let mat = Material::new("DEV/DEV_MEASUREWALL01C");
        let materials = &[&mat; 2];
        let num_sides = 32;
        map.add_solid(spike(
            &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
            // &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 512.0)),
            // &Bounds::new(Vector3::new(-8192.0, -8192.0, 0.0), Vector3::new(8192.0, 8192.0, 8192.0)),
            // &Bounds::new(Vector3::new(-16384.0, -16384.0, -16384.0), Vector3::new(16384.0, 16384.0, 16384.0)),
            // &Bounds::new(Vector3::new(-12288.0, -12288.0, -12288.0), Vector3::new(12288.0, 12288.0, 12288.0)),
            // &Bounds::new(Vector3::new(-32.0, -32.0, 0.0), Vector3::new(32.0, 32.0, 64.0)),
            // 64,
            num_sides,
            materials,
            &options,
        ));

        map.add_solid(spike(
            &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 256.0)),
            num_sides,
            materials,
            &options,
        ));

        map.add_solid(spike(
            &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
            num_sides,
            materials,
            &options,
        ));

        map.add_solid(spike(
            &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 1024.0)),
            num_sides,
            materials,
            &options,
        ));

        write_test_vmf(map.to_lower());

        // panic!()
    }

    #[ignore]
    #[test]
    fn cylinder_test() {
        dbg!();
        let mut map = Map::default();
        let options = SolidOptions::default();

        map.add_solid(cylinder(
            &Bounds::new(Vector3::new(-16.0, -16.0, 0.0), Vector3::new(16.0, 16.0, 32.0)),
            4,
            [&Material::new("DEV/DEV_MEASUREWALL01C"); 3],
            &options,
        ));

        write_test_vmf(map.to_lower());
    }

    fn write_test_vmf(vmf: Vmf<StrType<'_>>) {
        const OUTPUT_PATH: &str =
            "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2/custom/maps/output2.vmf";
        _ = std::fs::remove_file(OUTPUT_PATH);
        let mut output =
            std::fs::OpenOptions::new().write(true).create(true).open(OUTPUT_PATH).unwrap();

        use std::io::Write;
        writeln!(output, "{:#}", vmf).unwrap();
    }

    #[ignore]
    #[test]
    fn sphere_test() {
        dbg!();
        let mut map = Map::default();
        let options = SolidOptions{world_align: false, ..SolidOptions::default()};

        for solid in sphere(
            // &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
            &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 5120.0)),
            16,
            &[&Material::new("DEV/DEV_MEASUREWALL01C"); 1],
            &options,
        ) {
            map.add_solid(solid);
        }

        write_test_vmf(map.to_lower());

        panic!()
    }
}
