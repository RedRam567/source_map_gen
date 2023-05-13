#![allow(clippy::print_literal)] // false positive for file!()

use core::panic;
use std::{todo, dbg};

use super::*;
use crate::prelude::{Material, Plane, Side, Solid, Vector2};
use crate::{IterWithNext, OneOrVec};

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

// fn should_promote_frac(radius: f32, sides: u32) -> bool {
//     radius as u32 / 2 > sides
// }

// TODO: test non power of 2
/// returns true if it clamped the sides for the size of the circle.
/// also clamps for sides < 3
#[inline]
fn clamp_promote(radius: f32, sides: &mut u32, options: &SolidOptions) -> bool {
    // max sides before int rounding give 3 colinear points (2 coplanar planes)
    let clamped_sides = radius as u32 / 2;
    let mut changed = !options.allow_frac && options.frac_promote && (clamped_sides > *sides);
    if changed {
        *sides = clamped_sides;
    }
    // hackey felling
    if *sides < 3 {
        *sides = 3;
        changed = true;
    }
    changed
}

/// Returns an iterator to points on an ellipse starting north (+Y) and heading
/// east (+X, right, closewise) which is the same order Hammer seems to use.
/// All math is done internally as `f64` as an (in)sanity check.
///
/// See also <https://en.wikipedia.org/wiki/Ellipse>
pub fn ellipse_verts(
    center: Vector2<f32>, x_radius: f32, y_radius: f32, mut num_sides: u32, options: &SolidOptions,
) -> impl ExactSizeIterator<Item = Vector2<f32>> + Clone {
    assert!(x_radius > 0.0 && y_radius > 0.0, "cannot make ellipse of zero radius");
    // clamp for too small for sides and sides < 3
    let changed = clamp_promote((x_radius).min(y_radius), &mut num_sides, options);
    if changed {
        eprintln!("[{}:{}] Warning Ellipse: Sides clamped to {}. Colinear ellipse points, too small/too many sides. Ellipse(x:{:.1},y:{:.1})",
                file!(), line!(), num_sides, x_radius, y_radius);
    }

    // relative to north, right/clockwise(east)
    // RangeInclusive<u32> doesnt impl ExactSizeIterator BUT Range<u32> does
    // BUT neither range impl for 64/128 ints WTF???
    // https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html#implementors
    let allow_frac = options.allow_frac;
    let delta_angle = std::f64::consts::TAU / num_sides as f64;
    (0..num_sides).map(move |n| {
        let angle = delta_angle * (n + 1) as f64;
        let x = x_radius as f64 * -angle.cos() + center.x as f64;
        let y = y_radius as f64 * angle.sin() + center.y as f64;
        // Vector2::new(x as f32, y as f32)
        Vector2::new_with_round(x as f32, y as f32, allow_frac)
    })
}

// TODO: twisted circles and expand to bounds (circle with flat faces on aabb)
pub fn ellipse_verts_3d(
    center: Vector3<f32>, x_radius: f32, y_radius: f32, mut num_sides: u32, options: &SolidOptions,
) -> impl ExactSizeIterator<Item = Vector3<f32>> + Clone {
    // clamp for too small for sides and sides < 3
    let changed = clamp_promote((x_radius).min(y_radius), &mut num_sides, options);
    if changed {
        eprintln!("[{}:{}] Warning Ellipse: Sides clamped to {}. Colinear ellipse points, too small/too many sides. Ellipse(x:{:.1},y:{:.1})",
                file!(), line!(), num_sides, x_radius, y_radius);
    }

    // relative to north, right/clockwise(east)
    // RangeInclusive<u32> doesnt impl ExactSizeIterator BUT Range<u32> does
    // BUT neither range impl for 64/128 ints WTF???
    // https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html#implementors
    let allow_frac = options.allow_frac;
    let delta_angle = std::f64::consts::TAU / num_sides as f64;
    (0..num_sides).map(move |n| {
        let angle = delta_angle * (n + 1) as f64;
        let x = x_radius as f64 * -angle.cos() + center.x as f64;
        let y = y_radius as f64 * angle.sin() + center.y as f64;
        let z = center.z;
        // Vector2::new(x as f32, y as f32)
        Vector3::new_with_round(x as f32, y as f32, z, allow_frac)
    })
}

// let top = Plane::new(verts[4].const_clone(), verts[5].const_clone(), verts[6].const_clone()).with_texture(textures[0]);
// let bottom = Plane::new(verts[2].const_clone(), verts[1].const_clone(), verts[0].const_clone()).with_texture(textures[1]);
// let north = Plane::new(verts[2].const_clone(), verts[6].const_clone(), verts[5].const_clone()).with_texture(textures[2]);
// let south = Plane::new(verts[0].const_clone(), verts[4].const_clone(), verts[7].const_clone()).with_texture(textures[3]);
// let east = Plane::new(verts[3].const_clone(), verts[7].const_clone(), verts[6].const_clone()).with_texture(textures[5]);
// let west = Plane::new(verts[1].const_clone(), verts[5].const_clone(), verts[4].const_clone()).with_texture(textures[4]);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// How to split a shape into one or more solids.
pub enum Grouping {
    /// Automaticly choose between [`Grouping::Single`] or [`Grouping::Group`]
    #[default]
    Auto,
    /// A single [`Solid`]
    Single,
    /// Group into some number of groups. Ex: split a spike into four parts,
    /// split sphere into layers.
    Group,
    // /// Every face as seperate [`Solid`]
    // Face,
}

// include top
// include bottom
// is upside down cone -> auto? i mean how often we be adding non-cubes?
// A cylinder, frustum (truncated cone), cone or prism
// A
// NOTE: hammer cylinder order: bottom, top counter clock, sides in some order
/// A prism, cylinder, cone, frustum (truncated cone). Can be oblique (slanted)
/// and/or truncated (angled bases). Returns an iter of `Sides` in the order:
/// top base (if available), bottom base (if available), sides.
///
/// `top` and `bottom` are iters to points a polygon on a arbitrary plane which
/// are connected with faces.
/// `top` or `bottom` can also be a single point, at which it
/// will be interpreted as a cone and omit the apporpriate side.
/// `prefer_top` only matters for "weird" prisms, set to false if you dont know.
/// It controls wether the top or bottom base has two points associated with it
/// rather than one giving extra accuracy to vbsp/Hammer.
/// `prefer_top` is ignored by cones.
/// `mats` is an array of materals for the sides in the order:
/// top base, bottom base, sides.
///
/// # Panics
/// - `top` or `bottom` must least 3 length.
/// - `top` and `bottom` must not both be single points as that would be degenerate
///     and have no volume.
///
/// # Notes
/// - FIXME: When making ellipses bases or other weird stuff, unpreferred side will
///     seemingly be rotated clockwise by a bit for some reason. Maybe just
///     vbsp/Hammer's fault but maybe something wrong in this library.
/// - FIXME: BROKEN HANGS. If `top` and `bottom` are different lengths, it will advance both to the
///     shortest length (uses .zip())
/// - `top` and `bottom` must give their points in clockwise direction (ex: East from North).
/// - The first 3 points of `top` and `bottom` are used to make the base planes.
///     so non-planar bases will be truncated to planar.
/// - If `top` and `bottom` are both iters to single points. No special behavior
///     and it is treated as an upside down cone and will result in an invalid `Solid`
/// - Clamshells are possible, where the preferred base is a polygon and
///     unpreferred base is a line, but the unpreferred base side must be removed.
///     (with `.skip(1)` for example to skip the top base)
/// - Double clamshells seem to be impossible and almost-double clamshells
///     are extemely finicky and don't match porportions.
///
/// # Examples
/// ```rust
/// use proc_gen2::prelude::*;
/// use proc_gen2::map::Map;
/// use proc_gen2::generation2::shape::ellipse_verts_3d;
/// use proc_gen2::vmf::ToLower;
/// use proc_gen2::generation2::SolidOptions;
/// use proc_gen2::generation2::shape::prism;
///
/// let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
/// let mats = &[&dev_person; 3];
/// let options = &SolidOptions::default().allow_frac();
/// let mut map = Map::default();
///
/// // a perfect 512x512x512 cylinder
/// let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 32, options);
/// let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a 512x512x512 frustum
/// let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 256.0), 128.0, 128.0, 32, options);
/// let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a 512x512x512 cone
/// let top = std::iter::repeat(Vector3::new(0.0, 0.0, 256.0)).take(16);
/// let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 16, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a 512x512x512 upside down cone
/// let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 16, options);
/// let bottom = std::iter::repeat(Vector3::new(0.0, 0.0, -256.0)).take(16);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a weird shape
/// // try changing the false to true and see the top become a perfect circle
/// // while the bottom gets weird
/// let top = ellipse_verts_3d(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
/// let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// let vmf = map.to_lower();
/// println!("{}", vmf); // replace with writeln!() to a file
/// ```
///
/// ```rust ignore,no_run
/// // almost-double clamshell
/// let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
/// let mats = &[&dev_person; 3];
/// let options = &SolidOptions::default().allow_frac();
/// let mut map = Map::default();
/// // has 5120 x_radius but is only 1694 in hammer
/// let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 512.0), 5120.0, 0.0, 32, options);
/// // when 128 is smaller, shape bigger I think, and crashes vbsp and/or l4d2 doesnt load
/// // (Host_EndGame: Map coordinate extents are too large!!)
/// let bottom =
///     ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 128.0, 512.0, 32, options);
/// // NOTE: skip(0) or skip(1) seems to have no effect
/// let solid = Solid::new(prism(top, bottom, false, mats, options).skip(1).collect::<Vec<_>>());
/// map.add_solid(solid);
/// let vmf = map.to_lower();
/// // write!() to .vmf file here
/// ```
// TODO: rotate, split, stars, transform, sphere
pub fn prism<'a, I1, I2>(
    top: I1, bottom: I2, prefer_top: bool, mats: &'a [&'a Material<'a>; 3],
    options: &'a SolidOptions,
) -> impl Iterator<Item = Side<'a>> + Clone + 'a
where
    I1: Iterator<Item = Vector3<f32>> + Clone + 'a,
    I2: Iterator<Item = Vector3<f32>> + Clone + 'a,
{
    #[inline(always)] // eh
    fn iter_to_three<I, T>(mut iter: I) -> Option<[T; 3]>
    where
        I: Iterator<Item = T>,
    {
        Some([iter.next()?, iter.next()?, iter.next()?])
    }

    // Figure out if cone and choose different points to allow cones
    // TODO: check only 2 for performance, (maybe bad rounding tho)
    // OR distance check and collapse into 1 point (how choose?)
    // cant we just use base sides in next step then?
    // TODO: hmm, check for identical x,y, OR z for all?, to allow circle to line

    let is_top_one_point = {
        let mut top_clone = top.clone();
        let top_first = top_clone.next().expect("Must be at least 3 len");
        top_clone.all(|item| item == top_first)
    };
    let is_bottom_one_point = {
        let mut bottom_clone = bottom.clone();
        let bottom_first = bottom_clone.next().expect("Must be at least 3 len");
        bottom_clone.all(|item| item == bottom_first)
    };
    assert!(!(is_top_one_point && is_bottom_one_point), "Degenerate. Prism is a line");

    // Add top and bottom planes only if not a point
    // hammer just takes 3 adjacent points and does weird offset by 1 for bottom tho
    let top_side = (!is_top_one_point).then(|| {
        let [pt1, pt2, pt3] = iter_to_three(top.clone()).expect("Must be at least 3 len");
        Side::new_verts(pt1, pt2, pt3, mats[0], options)
    });
    let bottom_side = (!is_bottom_one_point).then(|| {
        let [pt1, pt2, pt3] = iter_to_three(bottom.clone()).expect("Must be at least 3 len");
        // NOTE: reversed order
        Side::new_verts(pt3, pt2, pt1, mats[1], options)
    });
    let top_bottom = top_side.into_iter().chain(bottom_side.into_iter());

    // Add sides
    let points4 = IterWithNext::new(top).zip(IterWithNext::new(bottom));
    let sides = points4.map(move |((top1, top2), (bottom1, bottom2))| {
        dbg!();
        // TODO: simplify. is_top/bottom should determine normal vs upside down
        // else prefer_top if both false (both will never be true because of assert)
        // is_bottom_one_point || prefer_top -> prefer top
        // is_top_one_point && prefer_top -> prefer bottom no matter what
        if (is_bottom_one_point || prefer_top) && !(is_top_one_point && prefer_top) {
            // upside down cone, prefer top, 2 points on top
            // NOTE: reversed order (not sure why, I thought was working fine but
            // its not ig)
            Side::new_verts(top2, top1, bottom1, mats[2], options)
        } else {
            // normal, upright cone, prefer bottom, 2 points on bottom
            Side::new_verts(bottom1, bottom2, top1, mats[2], options)
        }
    });

    top_bottom.chain(sides)
}

// TODO: sphere, arch, multi

// TODO:DOCS:
/// top, bottom, north, south, east, west
#[rustfmt::skip]
pub fn cube<'a>(bounds: &Bounds, materials: &[&Material<'a>; 6], options: &SolidOptions) -> Solid<'a> {
    let verts = bounds.verts();

    // let top = Side::new_parts_option(verts[SWT].clone(), verts[NWT].clone(), verts[NET].clone(), materials[0], options);
    // let top = Plane::new(verts[SWT].const_clone(), verts[NWT].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[0], align);
    // let bottom = Plane::new(verts[NEB].const_clone(), verts[NWB].const_clone(), verts[SWB].const_clone()).with_material_alignment(materials[1], align);
    // let north = Plane::new(verts[NEB].const_clone(), verts[NET].const_clone(), verts[NWT].const_clone()).with_material_alignment(materials[2], align);
    // let south = Plane::new(verts[SWB].const_clone(), verts[SWT].const_clone(), verts[SET].const_clone()).with_material_alignment(materials[3], align);
    // let east = Plane::new(verts[SEB].const_clone(), verts[SET].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[4], align);
    // let west = Plane::new(verts[NWB].const_clone(), verts[NWT].const_clone(), verts[SWT].const_clone()).with_material_alignment(materials[5], align);
    let top = Side::new_verts(verts[SWT].clone(), verts[NWT].clone(), verts[NET].clone(), materials[0], options);
    let bottom = Side::new_verts(verts[NEB].clone(), verts[NWB].clone(), verts[SWB].clone(), materials[1], options);
    let north = Side::new_verts(verts[NEB].clone(), verts[NET].clone(), verts[NWT].clone(), materials[2], options);
    let south = Side::new_verts(verts[SWB].clone(), verts[SWT].clone(), verts[SET].clone(), materials[3], options);
    let east = Side::new_verts(verts[SEB].clone(), verts[SET].clone(), verts[NET].clone(), materials[4], options);
    let west = Side::new_verts(verts[NWB].clone(), verts[NWT].clone(), verts[SWT].clone(), materials[5], options);

    Solid::new(vec![top, bottom, north, south, east, west])
}

// TODO:DOCS:
/// west/top slope, bottom, north, south, east
#[rustfmt::skip]
pub fn wedge<'a>(bounds: &Bounds, materials: &[&Material<'a>; 5], options: &SolidOptions) -> Solid<'a> {
    let verts = bounds.verts();

    // same as top but with first two verts on bottom
    // let slope = Plane::new(verts[SWB].const_clone(), verts[NWB].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[0], align);
    // let bottom = Plane::new(verts[NEB].const_clone(), verts[NWB].const_clone(), verts[SWB].const_clone()).with_material_alignment(materials[1], align);
    // let north = Plane::new(verts[NEB].const_clone(), verts[NET].const_clone(), verts[NWT].const_clone()).with_material_alignment(materials[2], align);
    // let south = Plane::new(verts[SWB].const_clone(), verts[SWT].const_clone(), verts[SET].const_clone()).with_material_alignment(materials[3], align);
    // let east = Plane::new(verts[SEB].const_clone(), verts[SET].const_clone(), verts[NET].const_clone()).with_material_alignment(materials[4], align);
    let slope = Side::new_verts(verts[SWB].clone(), verts[NWB].clone(), verts[NET].clone(), materials[0], options);
    let bottom = Side::new_verts(verts[NEB].clone(), verts[NWB].clone(), verts[SWB].clone(), materials[1], options);
    let north = Side::new_verts(verts[NEB].clone(), verts[NET].clone(), verts[NWT].clone(), materials[2], options);
    let south = Side::new_verts(verts[SWB].clone(), verts[SWT].clone(), verts[SET].clone(), materials[3], options);
    let east = Side::new_verts(verts[SEB].clone(), verts[SET].clone(), verts[NET].clone(), materials[4], options);

    Solid::new(vec![slope, bottom, north, south, east])
}

// top, down, "right"
// TODO: wibbly cone? arbitaray tip
/// bottom, sides
/// Uses same points as Hammer for the sides but different for the base.
/// # Note
/// Hammer (l4d2) seems to have a lot of problems with non-powers of two `num_sides` and
/// `bounds` that are roughly as tall as wide. Unless you know what you are doing, 8 seems to be the
/// highest without any problems at all although 16 is good for almost all.
/// Hammer has absolute limit of ~128 total faces
///
/// `vbsp` is much more forgiving and seems to allow all spikes with an absolute
/// max of 63 sides (64 total faces), contrary to the [Valve Wiki] which says 128.
///
/// [Valve Wiki]: https://developer.valvesoftware.com/wiki/Brush
pub fn spike<'a>(
    bounds: &Bounds, num_sides: u32, mats: &[&Material<'a>; 2], options: &SolidOptions,
) -> OneOrVec<Solid<'a>> {
    let top = bounds.max.z;
    let bottom = bounds.min.z;
    let center_xy = bounds.center_xy();
    let top_point = center_xy.with_z(top);

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
        sides.push(Side::new_verts(top_point.clone(), current, next, mats[1], options));
    }

    OneOrVec::One(Solid::new(sides))
}

// BUG: with all: small bug, bounds.top_plane... doesnt check allow_frac
/// top, bottom, sides
pub fn cylinder<'a>(
    bounds: &Bounds, num_sides: u32, mats: [&Material<'a>; 3], options: &SolidOptions,
) -> OneOrVec<Solid<'a>> {
    let top = bounds.max.z;
    let bottom = bounds.min.z;
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
        sides.push(Side::new_verts(current, next, top_point, mats[2], options));
    }

    OneOrVec::One(Solid::new(sides))
}

/// Top, Bottom, sides
/// A [`cylinder`] with different sized (and offset) bases. Bases must have the same number of sides
pub fn frustum_old<'a, I>(
    top: I, top_z: f32, bottom: I, bottom_z: f32, mats: &[&Material<'a>; 3], options: &SolidOptions,
) -> OneOrVec<Solid<'a>>
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
    for ((bottom1, bottom2), top1) in bottom_verts.zip(top) {
        sides.push(Side::new_verts(
            bottom1.with_z(bottom_z),
            bottom2.with_z(bottom_z),
            top1.with_z(top_z),
            mats[2],
            options,
        ));
    }

    OneOrVec::One(Solid::new(sides))
}

// get heights
// get "circumference"/"radius" of sphere at height
// bunch of frustums
// TODO: promote broken if caps use different numsides
/// inside, outside
pub fn sphere<'a>(
    bounds: &Bounds, num_sides: u32, mats: &[&Material<'a>; 2], options: &SolidOptions,
) -> OneOrVec<Solid<'a>> {
    let spike_mats = mats;
    let frustum_mats = &[mats[0], mats[0], mats[1]];

    let center_xy = bounds.center_xy();
    let center_z = bounds.center().z;
    let radius_x = bounds.x_len() / 2.0;
    let radius_y = bounds.y_len() / 2.0;

    // split sphere into equal height sections from the top
    // TODO: use angles instead or trig
    let relative_height = |n| {
        // 0.5 -> -0.5 as n -> num_sides
        let multiplier = (num_sides - n) as f32 / num_sides as f32 - 0.5;
        let height = center_z + multiplier * bounds.z_len();
        if options.allow_frac {
            height
        } else {
            height.round()
        }
    };
    let mut solids = OneOrVec::new();

    // make tops and bottoms
    let top_layer = relative_height(1);
    let bottom_layer = relative_height(num_sides);
    let top_xy = radius_at_sphere_height_xy(radius_x, radius_y, top_layer, options.allow_frac);
    let bottom_xy =
        radius_at_sphere_height_xy(radius_x, radius_y, bottom_layer, options.allow_frac);
    // NOTE: height absolute
    let top_bounds =
        Bounds::new(top_xy.with_z(top_layer + center_z), (-top_xy).with_z(bounds.max.z));
    let bottom_bounds =
        Bounds::new(bottom_xy.with_z(bottom_layer + center_z), (-bottom_xy).with_z(bounds.min.z));
    // solids.push_or_extend(spike(&top_bounds, num_sides, spike_mats, options));
    // solids.push_or_extend(spike(&bottom_bounds, num_sides, spike_mats, options)); // TODO: transform

    // make layers
    let heights = IterWithNext::new((1..num_sides).map(relative_height));
    for (height_top, height_bottom) in heights {
        let height_top_from_center = height_top - center_z;
        let height_bottom_from_center = height_bottom - center_z;

        // get radii
        let top_radius_x =
            radius_at_sphere_height(radius_x, height_top_from_center, options.allow_frac);
        let top_radius_y =
            radius_at_sphere_height(radius_y, height_top_from_center, options.allow_frac);
        let bottom_radius_x =
            radius_at_sphere_height(radius_x, height_bottom_from_center, options.allow_frac);
        let bottom_radius_y =
            radius_at_sphere_height(radius_y, height_bottom_from_center, options.allow_frac);

        // make circle bases
        let top_circle =
            ellipse_verts(center_xy.clone(), top_radius_x, top_radius_y, num_sides, options);
        let bottom_circle =
            ellipse_verts(center_xy.clone(), bottom_radius_x, bottom_radius_y, num_sides, options);

        // make frustum layer NOTE: height absolute
        let frustum = frustum_old(
            top_circle,
            height_top,
            bottom_circle,
            height_bottom,
            frustum_mats,
            options,
        );
        solids.push_or_extend(frustum);
    }
    solids
}

/// See <https://en.wikipedia.org/wiki/Circle_of_a_sphere>
fn radius_at_sphere_height(radius: f32, height_from_center: f32, allow_frac: bool) -> f32 {
    let height = height_from_center;
    let radius = f32::sqrt(radius * radius - height * height);
    if allow_frac {
        radius
    } else {
        radius.round()
    }
}

fn radius_at_sphere_height_xy(
    x_radius: f32, y_radius: f32, height_from_center: f32, allow_frac: bool,
) -> Vector2<f32> {
    let x = radius_at_sphere_height(x_radius, height_from_center, allow_frac);
    let y = radius_at_sphere_height(y_radius, height_from_center, allow_frac);
    Vector2::new(x, y)
}

#[cfg(test)]
mod tests {
    // /// Array that can be slice into a smaller sub-array
    // ///
    // /// Also see the [crate] level reference.
    // pub trait SubArray {
    //     /// The value type of this array.
    //     ///
    //     /// This is the `T` in `[T; N]` on regular arrays.
    //     type Item;

    //     /// Get a reference to a sub-array of length `N` starting at `offset`.
    //     ///
    //     /// # Panics
    //     /// Panics if `offset + N` exceeds the length of this array.
    //     ///
    //     /// # Example
    //     /// ```
    //     /// use sub_array::SubArray;
    //     ///
    //     /// let arr: [u8; 5] = [9, 8, 7, 6, 5];
    //     ///
    //     /// // Get a sub-array starting at offset 3
    //     /// let sub: &[u8; 2] = arr.sub_array_ref(3);
    //     /// assert_eq!(sub, &[6, 5]);
    //     /// ```
    //     fn sub_array_ref<const N: usize>(&self, offset: usize) -> &[Self::Item; N];

    //     /// Get a mutable reference to a sub-array of length `N` starting at
    //     /// `offset`.
    //     ///
    //     /// # Panics
    //     /// Panics if `offset + N` exceeds the length of this array.
    //     ///
    //     /// # Example
    //     /// ```
    //     /// use sub_array::SubArray;
    //     ///
    //     /// let mut arr: [u8; 5] = [9, 8, 7, 6, 5];
    //     ///
    //     /// // Get a mutable sub-array starting at offset 0
    //     /// let sub: &mut [u8; 2] = arr.sub_array_mut(0);
    //     /// assert_eq!(sub, &mut [9, 8]);
    //     /// ```
    //     fn sub_array_mut<const N: usize>(&mut self, offset: usize) -> &mut [Self::Item; N];
    // }

    // /// Implementation on regular arrays
    // impl<T, const M: usize> SubArray for [T; M] {
    //     type Item = T;

    //     fn sub_array_ref<const N: usize>(&self, offset: usize) -> &[Self::Item; N] {
    //         self[offset..(offset + N)].try_into().unwrap()
    //     }

    //     fn sub_array_mut<const N: usize>(&mut self, offset: usize) -> &mut [Self::Item; N] {
    //         (&mut self[offset..(offset + N)]).try_into().unwrap()
    //     }
    // }

    use vmf_parser_nom::ast::{Vmf, Property};

    use crate::map::Map;
    use crate::prelude::Vector3;
    use crate::vmf::ToLower;
    use crate::StrType;

    use self::panic;
    use super::*; // why tf is this nessessary?

    fn make_shape<'a>(
        shape: &str, bounds: &Bounds, sides: u32, mats: &[&Material<'a>; 6], options: &SolidOptions,
    ) -> OneOrVec<Solid<'a>> {
        // TODO: horrible
        // let spike_bounds = &Bounds {max: Vector3 {z: bounds.max.z / 2.0, ..bounds.max }, ..bounds.clone()};
        let spike_bounds = &bounds;
        match shape {
            "cube" => OneOrVec::One(cube(bounds, mats[..].try_into().unwrap(), options)),
            "wedge" => OneOrVec::One(wedge(bounds, mats[..5].try_into().unwrap(), options)),
            "spike" => spike(spike_bounds, sides, mats[..2].try_into().unwrap(), options),
            "cylinder" => cylinder(bounds, sides, mats[..3].try_into().unwrap(), options),
            // "frustum" => frustum(bounds, sides, mats[..].try_into().unwrap(), options),
            "sphere" => sphere(bounds, sides, mats[..2].try_into().unwrap(), options),
            str => panic!("unkown shape {}", str),
            // _ => OneOrVec::new()
        }
    }

    #[test]
    #[ignore]
    fn test_frustum_cone() {
        let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
        let mats = &[&dev_person; 3];
        let options = &SolidOptions::default().allow_frac();
        let mut map = Map::default();

        let top = ellipse_verts_3d(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
        let bottom =
            ellipse_verts_3d(Vector3::new(512.0, 512.0, -512.0), 256.0, 256.0, 32, options);
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 512.0), 1024.0, 0.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 256.0, 256.0, 32, options);
        let solid =
            Solid::new(prism(top, bottom, false, mats, options).skip(1).collect::<Vec<_>>());
        map.add_solid(solid);

        let mut map = Map::default();
        let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 512.0), 5120.0, 0.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 128.0, 512.0, 32, options);
        let solid =
            Solid::new(prism(top, bottom, false, mats, options).skip(1).collect::<Vec<_>>());
        map.add_solid(solid);

        write_test_vmf(map.to_lower());
        panic!("worked")
    }

    #[test]
    #[ignore]
    fn frustum_cone_doc_test() {
        let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
        let mats = &[&dev_person; 3];
        let options = &SolidOptions::default();
        let mut map = Map::default();
        map.options.cordon = Some(crate::generation::Bounds::new(Vector3::new(-5120.0, -5120.0, -5120.0), Vector3::new(5120.0, 5120.0, 5120.0)));
        // prevent FindPortalSide errors O_o
        map.defaults_l4d2();
        map.entities[0].props[0].value = "0 0 2048".into();
        dbg!(&map.entities[0].props[0]);

        // a perfect 512x512x512 cylinder
        let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        // a 512x512x512 frustum
        let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 256.0), 128.0, 128.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        // a 512x512x512 cone
        let top = std::iter::repeat(Vector3::new(0.0, 0.0, 256.0)).take(16);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 16, options);
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        // a 512x512x512 upside down cone
        let top = ellipse_verts_3d(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 16, options);
        let bottom = std::iter::repeat(Vector3::new(0.0, 0.0, -256.0));
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        // a weird shape
        // causes vbsp warnings if not enclosed and no enitities (others dont tho)
        let top = ellipse_verts_3d(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
        let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        // a weird shape with the bottom preferred NO WARNINGS? O_O?
        let top = ellipse_verts_3d(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
        let bottom = ellipse_verts_3d(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
        let solid = Solid::new(prism(top, bottom, true, mats, options).collect::<Vec<_>>());
        map.add_solid(solid);

        let vmf = map.to_lower();

        write_test_vmf(map.to_lower());
        panic!("worked")
    }

    #[test]
    #[ignore]
    fn shape_test() {
        // shape
        // size
        // size long
        // sides
        // let
        // 16x16 to 512

        let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
        let mats = [&dev_person; 6];

        const CELL_SIZE: i32 = 512;

        let shapes = ["cube", "wedge", "spike", "cylinder", "sphere"];
        // let shapes = ["cube", "wedge", "spike", "cylinder"];
        // let shapes = ["spike"];
        // let sides = [3, 4, 8, 16, 32, 63];
        // let sides = [3, 4, 8, 16, 32, 63];
        let sides = [3, 4, 8, 16];
        // let sizes = [16, 32, 64, 128, 256, 512];
        let sizes = [512, 256, 128, 64, 32, 16];
        let options = SolidOptions::default();

        let mut x = -1;
        let mut y = -1;
        let mut z = -1;

        // TODO: FIXME: REMEMBER TO ADD BACK SPHERE CAPS
        // TODO: align ignore pos
        let mut map = Map::default();
        for shape in shapes {
            for num_sides in sides {
                z = 0;
                for size in sizes {
                    // z += CELL_SIZE;
                    z += 0;
                    let x = (x + CELL_SIZE * 2 - size * 2) as f32;
                    let y = (CELL_SIZE / 2 + y - size / 2) as f32;
                    let z = (CELL_SIZE / 2 + z - size / 2) as f32;
                    let min = Vector3::new(x, y, z);
                    let max = min.clone() + size as f32;
                    let bounds = Bounds::new(min, max);
                    // map.add_solid(cube(&bounds, mats, &options));
                    // map.add_solid(wedge(&bounds, mats[..].try_into().unwrap(), &options));
                    map.add_solid2(make_shape(shape, &bounds, num_sides, &mats, &options));
                }
                x += CELL_SIZE * 2;
            }
            x = 0;
            y += CELL_SIZE;
        }

        write_test_vmf(map.to_lower());

        panic!("worked")
    }

    // // #[test]
    // // fn circle() {
    // //     // let truth = [];
    // //     let result = ellipse_verts(Vector3::default(), 64.0, 16);
    // //     dbg!(result.collect::<Vec<_>>());
    // //     panic!();
    // //     // for i in truth.iter().zip(result) {
    // //     //     // assert!(tu)
    // //     // }
    // // }

    // #[ignore]
    // #[test]
    // fn spike_test() {
    //     let mut map = Map::default();
    //     let options = SolidOptions::default();
    //     // let options = SolidOptions { allow_frac: false, ..Default::default() };
    //     // let options = SolidOptions { allow_frac: false, world_align: false };
    //     // let options = SolidOptions { allow_frac: true, ..Default::default() };

    //     // TODO: why cant I no longer do 127? or even 64?
    //     // WTF NOW I CANT EVEN USE 32 FROM HAMMER!!
    //     // I THINK 64 is vbsps limit (wiki says 128), I THINK 32 is hammers limit (fucking rip) -> split
    //     let mat = Material::new("DEV/DEV_MEASUREWALL01C");
    //     let materials = &[&mat; 2];
    //     let num_sides = 32;
    //     map.add_solid(spike(
    //         &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
    //         // &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 512.0)),
    //         // &Bounds::new(Vector3::new(-8192.0, -8192.0, 0.0), Vector3::new(8192.0, 8192.0, 8192.0)),
    //         // &Bounds::new(Vector3::new(-16384.0, -16384.0, -16384.0), Vector3::new(16384.0, 16384.0, 16384.0)),
    //         // &Bounds::new(Vector3::new(-12288.0, -12288.0, -12288.0), Vector3::new(12288.0, 12288.0, 12288.0)),
    //         // &Bounds::new(Vector3::new(-32.0, -32.0, 0.0), Vector3::new(32.0, 32.0, 64.0)),
    //         // 64,
    //         num_sides,
    //         materials,
    //         &options,
    //     ));

    //     map.add_solid(spike(
    //         &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 256.0)),
    //         num_sides,
    //         materials,
    //         &options,
    //     ));

    //     map.add_solid(spike(
    //         &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
    //         num_sides,
    //         materials,
    //         &options,
    //     ));

    //     map.add_solid(spike(
    //         &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 1024.0)),
    //         num_sides,
    //         materials,
    //         &options,
    //     ));

    //     write_test_vmf(map.to_lower());

    //     // panic!()
    // }

    // #[ignore]
    // #[test]
    // fn cylinder_test() {
    //     dbg!();
    //     let mut map = Map::default();
    //     let options = SolidOptions::default();

    //     map.add_solid(cylinder(
    //         &Bounds::new(Vector3::new(-16.0, -16.0, 0.0), Vector3::new(16.0, 16.0, 32.0)),
    //         4,
    //         [&Material::new("DEV/DEV_MEASUREWALL01C"); 3],
    //         &options,
    //     ));

    //     write_test_vmf(map.to_lower());
    // }

    fn write_test_vmf(vmf: Vmf<StrType<'_>>) {
        const OUTPUT_PATH: &str =
            "/home/redram/.local/share/Steam/steamapps/common/Left 4 Dead 2/custom/maps/output2.vmf";
        _ = std::fs::remove_file(OUTPUT_PATH);
        let mut output =
            std::fs::OpenOptions::new().write(true).create(true).open(OUTPUT_PATH).unwrap();

        use std::io::Write;
        writeln!(output, "{:#}", vmf).unwrap();
    }

    // #[ignore]
    // #[test]
    // fn sphere_test() {
    //     dbg!();
    //     let mut map = Map::default();
    //     let options = SolidOptions { world_align: false, ..SolidOptions::default() };

    //     for solid in sphere(
    //         // &Bounds::new(Vector3::new(-256.0, -256.0, 0.0), Vector3::new(256.0, 256.0, 512.0)),
    //         &Bounds::new(Vector3::new(-2560.0, -2560.0, 0.0), Vector3::new(2560.0, 2560.0, 5120.0)),
    //         16,
    //         &[&Material::new("DEV/DEV_MEASUREWALL01C"); 1],
    //         &options,
    //     ) {
    //         map.add_solid(solid);
    //     }

    //     write_test_vmf(map.to_lower());

    //     panic!()
    // }
}
