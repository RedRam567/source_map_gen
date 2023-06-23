use super::radius_at_sphere_height;
use super::*;

// use super::radius_at_sphere_height_xy;

fn radius_at_sphere_height_xy(
    x_radius: f32, y_radius: f32, height_from_center: f32, allow_frac: bool,
) -> Vector2<f32> {
    let x = radius_at_sphere_height(x_radius, height_from_center, allow_frac);
    let y = radius_at_sphere_height(y_radius, height_from_center, allow_frac);
    Vector2::new(x, y)
}

use super::ellipse_verts_2d;
use crate::prelude::{Material, Plane, Side, Solid, Vector2};
use crate::utils::{IterWithNext, OneOrVec};
use std::dbg;

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
    let circle_verts = IterWithNext::new(ellipse_verts_2d(
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
    let circle_verts = IterWithNext::new(ellipse_verts_2d(
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
            ellipse_verts_2d(center_xy.clone(), top_radius_x, top_radius_y, num_sides, options);
        let bottom_circle = ellipse_verts_2d(
            center_xy.clone(),
            bottom_radius_x,
            bottom_radius_y,
            num_sides,
            options,
        );

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
