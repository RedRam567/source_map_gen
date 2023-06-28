// mod old;

use super::*;
use crate::generation2::disp::{project_cube_to_sphere, Displacement};
use crate::prelude::{Material, Side, Solid};
use crate::utils::IterWithNext;
use crate::utils::Vec2d;

// TODO: solid transform

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

// TODO: twisted circles and expand to bounds (circle with flat faces on aabb)
pub fn ellipse_verts(
    center: Vector3<f32>, x_radius: f32, y_radius: f32, mut sides: u32, options: &SolidOptions,
) -> impl ExactSizeIterator<Item = Vector3<f32>> + Clone {
    // clamp for too small for sides and sides < 3
    let changed = clamp_promote((x_radius).min(y_radius), &mut sides, options);
    if changed {
        eprintln!("[{}:{}] Warning Ellipse: Sides clamped to {}. Colinear ellipse points, too small/too many sides. Ellipse(x:{:.1},y:{:.1})",
                file!(), line!(), sides, x_radius, y_radius);
    }

    // relative to north, right/clockwise(east)
    // RangeInclusive<u32> doesnt impl ExactSizeIterator BUT Range<u32> does
    // BUT neither range impl for 64/128 ints WTF???
    // https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html#implementors
    let allow_frac = options.allow_frac;
    let delta_angle = std::f64::consts::TAU / sides as f64;
    (0..sides).map(move |n| {
        let angle = delta_angle * (n + 1) as f64;
        let x = x_radius as f64 * -angle.cos() + center.x as f64;
        let y = y_radius as f64 * angle.sin() + center.y as f64;
        let z = center.z;
        // Vector2::new(x as f32, y as f32)
        Vector3::new_with_round(x as f32, y as f32, z, allow_frac)
    })
}

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

// NOTE: hammer cylinder order: bottom, top counter clock, sides in some order
// TODO: rotate, split, stars, transform, sphere
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
/// - If `top` and `bottom` are different lengths, it will advance both to the
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
/// use proc_gen2::generation2::shape::ellipse_verts;
/// use proc_gen2::vmf::ToLower;
/// use proc_gen2::generation2::SolidOptions;
/// use proc_gen2::generation2::shape::prism;
///
/// let dev_person = Material::new("DEV/DEV_MEASUREWALL01C");
/// let mats = [&dev_person; 3];
/// let options = &SolidOptions::default().allow_frac();
/// let mut map = Map::default();
///
/// // a perfect 512x512x512 cylinder
/// let top = ellipse_verts(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 32, options);
/// let bottom = ellipse_verts(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a 512x512x512 frustum
/// let top = ellipse_verts(Vector3::new(0.0, 0.0, 256.0), 128.0, 128.0, 32, options);
/// let bottom = ellipse_verts(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 32, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a 512x512x512 cone
/// let top = std::iter::repeat(Vector3::new(0.0, 0.0, 256.0));
/// let bottom = ellipse_verts(Vector3::new(0.0, 0.0, -256.0), 256.0, 256.0, 16, options);
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a 512x512x512 upside down cone
/// let top = ellipse_verts(Vector3::new(0.0, 0.0, 256.0), 256.0, 256.0, 16, options);
/// let bottom = std::iter::repeat(Vector3::new(0.0, 0.0, -256.0));
/// let solid = Solid::new(prism(top, bottom, false, mats, options).collect::<Vec<_>>());
/// map.add_solid(solid);
///
/// // a weird shape
/// // try changing the false to true and see the top become a perfect circle
/// // while the bottom gets weird
/// // NOTE: this shape makes vbsp make FindPortalSide errors for some reason
/// // but those go away if you enclose the map and add an entity
/// let top = ellipse_verts(Vector3::new(512.0, 512.0, 512.0), 256.0, 256.0, 32, options);
/// let bottom = ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 1024.0, 512.0, 32, options);
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
/// let top = ellipse_verts(Vector3::new(0.0, 0.0, 512.0), 5120.0, 0.0, 32, options);
/// // when 128 is smaller, shape bigger I think, and crashes vbsp and/or l4d2 doesnt load
/// // (Host_EndGame: Map coordinate extents are too large!!)
/// let bottom =
///     ellipse_verts(Vector3::new(0.0, 0.0, 0.0), 128.0, 512.0, 32, options);
/// // NOTE: skip(0) or skip(1) seems to have no effect
/// let solid = Solid::new(prism(top, bottom, false, mats, options).skip(1).collect::<Vec<_>>());
/// map.add_solid(solid);
/// let vmf = map.to_lower();
/// // write!() to .vmf file here
/// ```
pub fn prism<'a, I1, I2>(
    top: I1, bottom: I2, prefer_top: bool, mats: [&'a Material<'a>; 3], options: &'a SolidOptions,
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

    // Add top and bottom planes only if not a point
    // NOTE: (only checks 3 points for performance and not hanging when given an endless iterator)
    // hammer just takes 3 adjacent points and does weird offset by 1 for bottom tho
    let [pt1, pt2, pt3] = iter_to_three(top.clone()).expect("Must be at least 3 len");
    let is_top_one_point = pt1 == pt2 && pt2 == pt3;
    let top_side = (!is_top_one_point).then_some(Side::new_verts(pt1, pt2, pt3, mats[0], options));

    let [pt1, pt2, pt3] = iter_to_three(bottom.clone()).expect("Must be at least 3 len");
    let is_bottom_one_point = pt1 == pt2 && pt2 == pt3; // NOTE: reversed order
    let bottom_side =
        (!is_bottom_one_point).then_some(Side::new_verts(pt3, pt2, pt1, mats[1], options));

    assert!(!(is_top_one_point && is_bottom_one_point), "Degenerate. Prism is a line");

    let top_bottom = top_side.into_iter().chain(bottom_side.into_iter());

    // Add sides
    let points4 = IterWithNext::new(top).zip(IterWithNext::new(bottom));
    let sides = points4.map(move |((top1, top2), (bottom1, bottom2))| {
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
    let slope = Side::new_verts(verts[SWB].clone(), verts[NWB].clone(), verts[NET].clone(), materials[0], options);
    let bottom = Side::new_verts(verts[NEB].clone(), verts[NWB].clone(), verts[SWB].clone(), materials[1], options);
    let north = Side::new_verts(verts[NEB].clone(), verts[NET].clone(), verts[NWT].clone(), materials[2], options);
    let south = Side::new_verts(verts[SWB].clone(), verts[SWT].clone(), verts[SET].clone(), materials[3], options);
    let east = Side::new_verts(verts[SEB].clone(), verts[SET].clone(), verts[NET].clone(), materials[4], options);

    Solid::new(vec![slope, bottom, north, south, east])
}

// TODO: verify Uses same points as Hammer for the sides but different for the base.
/// A cone or spike with a base conected with triangles to a point.
/// A simple wrapper around [`prism()`].
///
/// `mats` are the materials in the order: base, sides.
///
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
#[doc(alias = "cone")]
pub fn spike<'a>(
    bounds: &Bounds, sides: u32, mats: [&'a Material<'a>; 2], options: &'a SolidOptions,
) -> Solid<'a> {
    let x_radius = bounds.x_len() / 2.0;
    let y_radius = bounds.y_len() / 2.0;
    let top_points = std::iter::repeat(bounds.top_center());
    let bottom_points = ellipse_verts(bounds.bottom_center(), x_radius, y_radius, sides, options);

    // NOTE: this is why `mats` is owned array of refs in `prism()`
    // lifetime problems with reference to this new array
    let mats = [mats[0], mats[0], mats[1]];
    // all the work is done here
    Solid::new(prism(top_points, bottom_points, false, mats, options).collect())
}
/// A cylinder. two bases connected with planes. A simple wrapper around [`prism()`].
/// `mats` are the materials in the order: top, bottom, sides.
///
/// # Notes
/// Recomened max sides is 32. 62 should work tho.
// TODO: 63 also seems to work (in sphere)??????
pub fn cylinder<'a>(
    bounds: &Bounds, sides: u32, mats: [&'a Material<'a>; 3], options: &'a SolidOptions,
) -> Solid<'a> {
    let x_radius = bounds.x_len() / 2.0;
    let y_radius = bounds.y_len() / 2.0;
    let top_points = ellipse_verts(bounds.top_center(), x_radius, y_radius, sides, options);
    let bottom_points = ellipse_verts(bounds.bottom_center(), x_radius, y_radius, sides, options);

    // NOTE: prefer_top in solid_options shouldn't matter as its all nice geometry
    Solid::new(prism(top_points, bottom_points, false, mats, options).collect())
}

// TODO: layer sides and side sides for nice square faces sphere
// TODO:FEATURE: add prism support for len and len*2 iters for nice caps in Hammer
// https://en.wikipedia.org/wiki/Square_antiprism
/// A sphere with topology similar to a globe.
///
/// Made of layers of frustums with with top and bottom being cones.
/// `mats` are the materials in the order: top insides, bottom insides, sides
///
/// # Notes
/// - [`sphere()`] is recomended over this.
/// - Recomened max sides is 8 else the top and bottom cones start to have problems
///     rendering in Hammer (vbsp seems fine tho).
/// - Max sides is ~120 for Hammer and 62 for vbsp. TODO: why does 63 work?
/// - Also shares limitations of [`spike()`], [`cylinder()`], and [`prism()`].
/// - FIXME: breaks if sphere is too small and [`ellipse_verts`] clamps number of
///     bases for top and bottom cones. `allow_frac` or `frac_promote` fixes this
pub fn sphere_globe<'a>(
    bounds: &Bounds, sides: u32, mats: [&'a Material<'a>; 3], options: &'a SolidOptions,
) -> Vec<Solid<'a>> {
    let x_radius = bounds.x_len() / 2.0;
    let y_radius = bounds.y_len() / 2.0;
    let z_radius = bounds.z_len() / 2.0;
    let center = bounds.center();

    // get height of point on sphere from top to bottom at constant angle
    let delta_angle = std::f64::consts::TAU / sides as f64;
    let height_at_angles = (0..=sides).map(move |n| {
        let angle = delta_angle * n as f64 / 2.0;
        let z = z_radius as f64 * angle.cos();
        if options.allow_frac {
            z as f32
        } else {
            (z as f32).round()
        }
    });

    // TODO: impl DoubleEndedIterator for IterWithNext
    // Iter from up to down of top AND bottom circle heights
    let heights = height_at_angles.collect::<Vec<_>>();
    let heights = heights.windows(2);
    let layers = heights.map(|height| {
        let height_top_from_center = height[0];
        let height_bottom_from_center = height[1];

        // get radii for xy of top and bottom
        let top_radius_x =
            radius_at_sphere_height(x_radius, height_top_from_center, options.allow_frac);
        let top_radius_y =
            radius_at_sphere_height(y_radius, height_top_from_center, options.allow_frac);
        let bottom_radius_x =
            radius_at_sphere_height(x_radius, height_bottom_from_center, options.allow_frac);
        let bottom_radius_y =
            radius_at_sphere_height(y_radius, height_bottom_from_center, options.allow_frac);

        // make top and bottom circles/points
        let top_center = Vector3 { z: height_top_from_center, ..center };
        let bottom_center = Vector3 { z: height_bottom_from_center, ..center };

        let top_circle = ellipse_verts(top_center, top_radius_x, top_radius_y, sides, options);
        let bottom_circle =
            ellipse_verts(bottom_center, bottom_radius_x, bottom_radius_y, sides, options);

        // TODO: allow choosing prefer top/bottom/auto
        Solid::new(prism(top_circle, bottom_circle, false, mats, options).collect::<Vec<_>>())
    });

    layers.collect()
}

/// See <https://en.wikipedia.org/wiki/Circle_of_a_sphere>
fn radius_at_sphere_height(radius: f32, height_from_center: f32, allow_frac: bool) -> f32 {
    // make sure that top/bottom of sphere is actually one point
    const EPSILON: f32 = 0.25; // arbitrary
    if (radius - height_from_center).abs() < EPSILON {
        return 0.0;
    }

    let height = height_from_center;
    let radius = f32::sqrt(radius * radius - height * height);
    if allow_frac {
        radius
    } else {
        radius.round()
    }
}

// TODO: automatic texturing
// 512x1024x128 ex: top xy=.25*8/6, side y=.25/2, side2 y=.25/3
// largest: measure "corners", use closest fraction
// sides: idk, need to be "relative" to largest side
// NOTE: The displacement uses `normals` and
// `distances` VMF fields rather than `offsets` so lighting shouldn't be weird TODO: actually verify

/// A displacement cube projected into a sphere.
///
/// # Notes
/// You will have to adjust the texturing on ellipsoids manually to look better.
/// For best results, make a block brush of the same dimensions. Match the
/// largest face of the ellipsoid to the scale of the block (0.25 * FRACTION)
/// and copy paste to the other faces. Adjust the other faces' X or Y in the
/// same manner (0.25 * MORE FRACTIONS).
///
/// See also: [`sphere_globe()`]
pub fn sphere<'a>(
    bounds: &Bounds, mut power: u32, mats: [&'a Material<'a>; 1], options: &'a SolidOptions,
) -> Solid<'a> {
    if power < 2 {
        // NOTE: hammer seems to support power 1 displacements O_O
        // bsp seems to definitely not support it
        eprintln!("[sphere()] power clamped to 2");
        power = 1;
    } else if power > 4 {
        eprintln!("[sphere()] power clamped to 4");
        power = 4;
    };
    let size = Displacement::power_to_len(power);

    let mut cube = cube(bounds, &[mats[0]; 6], options);

    for side in cube.sides.iter_mut() {
        // create a displacement side with points in default pos
        let mut disp = Displacement::new_plane(side.plane.clone(), size);
        let ideal = disp.ideal_points();

        // Project each point
        let projected = ideal.inner.iter().map(|p| {
            // convert to unit coords, project, convert back to real coords
            let unit = bounds.point_to_unit(p);
            let sphere_point = project_cube_to_sphere(&unit);
            bounds.unit_to_point(&sphere_point)
        });

        // convert ideal and projected to dir and dists
        let mut dirs = Vec2d::new(Vec2d::strides(size));
        let mut dists = Vec2d::new(Vec2d::strides(size));
        let alphas = Vec2d::from_parts(vec![0.0; disp.width * disp.width], Vec2d::strides(size));
        for (ideal, projected) in ideal.inner.iter().zip(projected) {
            // let (mut dir, dist) = ideal.dir_and_dist(&projected);
            let (mut dir, dist) = ideal.dir_and_dist(&projected);
            // remove nans TODO:DOCS: why it make them? cant explain, too tired rn
            if dir.x.is_nan() {
                // eprintln!("DIR IS NAN: {}", dir);
                dir = Vector3::origin();
            }
            dirs.inner.push(dir);
            dists.inner.push(dist);
        }
        disp.normals = dirs;
        disp.distances = dists;
        disp.alphas = alphas;

        side.disp = Some(disp);
    }

    cube
}

// // Scale a point in a bounds to a unit vector (-1..=1 on all axes)
// fn bounds_point_to_unit(bounds: &Bounds, point: &Vector3<f32>) -> Vector3<f32> {
//     let x = remap(point.x, bounds.x_range(), -1f32..1f32);
//     let y = remap(point.y, bounds.y_range(), -1f32..1f32);
//     let z = remap(point.z, bounds.z_range(), -1f32..1f32);

//     Vector3::new(x, y, z)
// }

// /// Scale a unit vector (-1..=1 on all axes) to bounds coords.
// fn bounds_unit_to_point(bounds: &Bounds, unit: &Vector3<f32>) -> Vector3<f32> {
//     let x = remap(unit.x, -1f32..1f32, bounds.x_range());
//     let y = remap(unit.y, -1f32..1f32, bounds.y_range());
//     let z = remap(unit.z, -1f32..1f32, bounds.z_range());

//     Vector3::new(x, y, z)
// }

#[test]
#[cfg(test)]
#[rustfmt::skip]
fn remap_test() {
    const EPSILON: f32 = 1e-4;

    fn assert(truth: f32, value: f32, from_range: Range<f32>, to_range: Range<f32>) {
        // range order doesn't matter at all
        assert_inner(truth, value, from_range.clone(), to_range.clone());
        let to_range = to_range.start..to_range.end;
        assert_inner(truth, value, from_range.clone(), to_range.clone());
        let from_range = from_range.start..from_range.end;
        assert_inner(truth, value, from_range.clone(), to_range.clone());
        let to_range = to_range.start..to_range.end;
        assert_inner(truth, value, from_range, to_range);
    }
    fn assert_reflexive(value: f32, from_range: Range<f32>, to_range: Range<f32>) {
        let truth = remap(value, from_range.clone(), to_range.clone());
        assert(truth, value, from_range, to_range);
    }
    fn assert_inner(truth: f32, value: f32, from_range: Range<f32>, to_range: Range<f32>) {
        let output_value = remap(truth, to_range.clone(), from_range.clone());
        assert!(
            (value - output_value).abs() < EPSILON,
            "truth: {truth}, result: {output_value}, value: {value}, from_range: {from_range:?}, to_range: {to_range:?}"
        );

        let output_truth = remap(value, from_range.clone(), to_range.clone());
        assert!(
            (truth - output_truth).abs() < EPSILON,
            "truth: {truth}, result: {output_truth}, value: {value}, from_range: {from_range:?}, to_range: {to_range:?}"
        );
    }

    // test(123.0, 1237.0, 1.0..1.0,        1f32..1f32); // gives nan

    assert(-1.0,  0.0,    0.0..1.0,       -1f32..1f32);
    assert(-0.5,  0.25,   0.0..1.0,       -1f32..1f32);
    assert(0.0,   0.5,    0.0..1.0,       -1f32..1f32);
    assert(0.5,   0.75,   0.0..1.0,       -1f32..1f32);
    assert(1.0,   1.0,    0.0..1.0,       -1f32..1f32);
    assert(-1.0,  -200.0, -200.0..200.0,  -1f32..1f32);
    assert(0.0,   0.0,    -200.0..200.0,  -1f32..1f32);
    assert(1.0,   200.0,  -200.0..200.0,  -1f32..1f32);
    assert(-1.0,  -100.0, -100.0..200.0,  -1f32..1f32);
    assert(0.0,   50.0,   -100.0..200.0,  -1f32..1f32);
    assert(1.0,   200.0,  -100.0..200.0,  -1f32..1f32);
    
    assert_reflexive(-1.0,    -200.0..200.0,          -1f32..9f32);
    assert_reflexive(0.0,     -100.0..200.0,          -2f32..8f32);
    assert_reflexive(-0.5,    69.0..200.0,            -3f32..7f32);
    assert_reflexive(0.25,    1212313.0..9212313.0,   -4f32..6f32);
    assert_reflexive(0.0,     0.0..1.0,               -5f32..5f32);
    assert_reflexive(0.5,     0.0..1.0,               -6f32..4f32);
    assert_reflexive(0.5,     0.0..1.0,               -7f32..3f32);
    assert_reflexive(0.75,    0.0..1.0,               -8f32..2f32);
    assert_reflexive(1.0,     0.0..1.0,               -9f32..1f32);
    assert_reflexive(1.0,     0.0..1.0,               10f32..1f32);
    assert_reflexive(-1.0,    -200.0..200.0,          -1f32..1f32);
    assert_reflexive(-200.0,  -200.0..200.0,          -1f32..1f32);
    assert_reflexive(0.0,     -200.0..200.0,          -1f32..1f32);
    assert_reflexive(0.0,     -200.0..200.0,          -1f32..1f32);
    assert_reflexive(1.0,     -200.0..200.0,          -1f32..1f32);
    assert_reflexive(200.0,   -200.0..200.0,          -1f32..1f32);
    assert_reflexive(-1.0,    -100.0..200.0,          -1f32..1f32);
    assert_reflexive(-100.0,  -100.0..200.0,          -1f32..1f32);
    assert_reflexive(0.0,     -100.0..200.0,          -1f32..1f32);
    assert_reflexive(50.0,    -100.0..200.0,          -1f32..1f32);
    assert_reflexive(1.0,     -100.0..200.0,          -1f32..1f32);
    assert_reflexive(200.0,   -100.0..200.0,          -1f32..1f32);
    // seems to have trouble with this
    // test_reflexive(0.25,    1212313.0..92123112893.0, -4f32..6f32);

}

// #[test]
// #[cfg(test)]
// fn unit() {
//     fn test(truth: f32, bottom: f32, top: f32, value: f32) {
//         assert_eq!(truth, scale_to_unit(bottom, top, value));
//         assert_eq!(value, disp::lerp(bottom, top, unit_to_multiplier(truth)));
//     }
//     test(-1.0, 0.0, 1.0, 0.0);
//     test(-0.5, 0.0, 1.0, 0.25);
//     test(0.0, 0.0, 1.0, 0.5);
//     test(0.5, 0.0, 1.0, 0.75);
//     test(1.0, 0.0, 1.0, 1.0);
//     test(-1.0, -200.0, 200.0, -200.0);
//     test(0.0, -200.0, 200.0, 0.0);
//     test(1.0, -200.0, 200.0, 200.0);
//     test(-1.0, -100.0, 200.0, -100.0);
//     test(0.0, -100.0, 200.0, 50.0);
//     test(1.0, -100.0, 200.0, 200.0);
// }

#[cfg(test)]
mod tests;
