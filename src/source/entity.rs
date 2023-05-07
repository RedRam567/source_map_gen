//! Contains [`PointEntity`] and other Source Engine Entities.

use std::fmt::Display;

use rgb::RGB8;
use vmf_parser_nom::ast::Property;

use crate::{
    map::Entity,
    source::{Angles, ColorBrightness},
    prelude::Vector3,
    vmf::ToLower,
};

// format keyvalues on valve dev wiki to doc comments:
// xclip -o -sel clip | sed ':a;N;$!ba;s/\n    //g' | sed '/^$/d' | sed ':a;N;$!ba;s/\n)/)/g' | sed -e 's_^_/// _' -e 's/<.*>//g' -e 's/ (in all games since)//g' -e 's/(/`(/' -e 's/)/)`/'

/// Push strtype if not empty.
fn props_push_string<'a, S>(props: &mut Vec<Property<S, S>>, key: &'a str, value: S)
where
    S: AsRef<str> + From<&'a str>,
{
    if !value.as_ref().is_empty() {
        props.push(Property::new(key, value));
    }
}

/// Push value if not default and convert to string.
fn props_push_value<'a, S, T>(props: &mut Vec<Property<S, S>>, key: &'a str, value: T)
where
    S: AsRef<str> + From<&'a str> + From<String>,
    T: Default + Display + PartialEq,
{
    if value != T::default() {
        props.push(Property::new(key, value.to_string()));
    }
}

// TODO:DOCS:
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointEntity<S> {
    /// Name `(targetname)` The name that other entities use to refer to this entity.
    pub name: S,
    /// Parent `(parentname)` Maintain the same initial offset to this entity. An attachment point can also be used if separated by a comma at the end. (parentname [targetname],[attachment])
    /// Tip: Entities transition to the next map with their parents
    /// Tip: phys_constraint can be used as a workaround if parenting fails.
    ///
    /// See also: <https://developer.valvesoftware.com/wiki/Targetname>
    pub parent: S,
    /// Origin `(X Y Z)` (origin) The position of this entity's center in the world. Rotating entities typically rotate around their origin.
    /// Note: Hammer does not move the entities accordingly only in the editor.
    pub origin: Vector3<f64>,
    /// Pitch Yaw Roll `(X Y Z)` (angles) This entity's orientation in the world. Pitch is rotation around the Y axis, yaw is the rotation around the Z axis, roll is the rotation around the X axis.
    /// Note: This works on brush entities, although Hammer doesn't show the new angles.
    pub angles: Angles,
    /// Classname `(classname)` Determines the characteristics of the entity before it spawns.
    /// Tip: Changing this on runtime still has use, like making matching an entry in S_PreserveEnts will persist the entity on new rounds!
    pub classname: S,
    /// Flags `(spawnflags)` Toggles exclusive features of an entity, its specific number is determined by the combination of flags added.
    pub flags: i32,
    /// Effects `(effects)` Combination of effect flags to use.
    pub effects: i32,
    /// Entity Scripts `(vscripts)` Space delimited list of VScript files (without file extension) that are executed after all entities have spawned. The scripts are all executed in the same script scope, later ones overwriting any identical variables and functions. Scripts executed on the worldspawn entity will be placed in root scope.
    pub vscripts: S,
    /// Think function `(thinkfunction)` Name of the function within this entity's script that'll be called automatically every 100 milliseconds, or a user-defined interval if the function returns a number. Avoid expensive operations in this function, as it may cause performance problems.
    pub vthink_fn: S,
    /// Lag Compensation `(LagCompensate)` Set to Yes to lag compensate this entity. Should be used very sparingly!
    pub lag_compensate: bool,
    /// Is Automatic-Aim Target `(is_autoaim_target)` If set to 1, this entity will slow down aiming movement for consoles and joystick controllers when the entity is under the crosshairs.
    pub is_autoaim_target: bool,
}

// from str and String for cow,
impl<'a, S> ToLower<Entity<S>> for PointEntity<S>
where
    S: AsRef<str> + Clone + From<&'a str> + From<String>,
{
    fn into_lower(self) -> Entity<S> {
        let mut props_v = Vec::with_capacity(11);
        let props = &mut props_v;

        props_push_string(props, "targetname", self.name);
        props_push_string(props, "parentname", self.parent);
        props_push_value(props, "origin", self.origin); // TODO: always
        props_push_value(props, "angles", self.angles);
        props_push_string(props, "classname", self.classname); // TODO: always
        props_push_value(props, "spawnflags", self.flags);
        props_push_value(props, "effects", self.effects);
        props_push_string(props, "vscripts", self.vscripts);
        props_push_string(props, "thinkfunction", self.vthink_fn);
        props_push_value(props, "LagCompensate", "1"); // TODO: alloc
        props_push_value(props, "is_autoaim_target", "1");

        Entity::new(props_v)
    }
}

// TODO: generic entity

// TODO:DOCS:
/// See also: <https://developer.valvesoftware.com/wiki/Light_environment>
#[derive(Clone, Debug, PartialEq)]
pub struct LightEnviroment<S> {
    /// Common Fields. Including classname, origin, angles, flags, etc.
    pub point_entity: PointEntity<S>,

    /// Pitch `(pitch)`. Overrides the pitch value in Angles, even if left at 0, so it needs to be specified. Contrary to Angles, the rotation of this pitch is measured counter-clockwise from the horizontal, so that 90 is straight up, while -90 is straight down. (It's simply the negative of a normal pitch value.)
    pub pitch: Option<f64>,
    /// Brightness `(_light)`. Color and brightness of the direct sunlight.
    pub direct_color: ColorBrightness,
    /// Ambient `(_ambient)`. Color and brightness of the diffuse skylight.
    pub amb_color: ColorBrightness,

    /// BrightnessHDR `(_lightHDR)`
    /// Override for Brightness when compiling HDR lighting. Defaults to -1 -1 -1 1, which means "same as LDR".
    pub direct_color_hdr: Option<ColorBrightness>,
    /// BrightnessScaleHDR `(_lightscaleHDR)` Amount to scale the direct light by when compiling for HDR.
    pub direct_hdr_scale: f64,

    /// AmbientHDR (_ambientHDR) Override for Ambient when compiling HDR lighting. Defaults to -1 -1 -1 1, which means "same as LDR".
    pub amb_color_hdr: Option<ColorBrightness>,
    /// AmbientScaleHDR (_AmbientScaleHDR) Amount to scale the ambient light by when compiling for HDR.
    pub amb_hdr_scale: f64,

    /// SunSpreadAngle (SunSpreadAngle) The angular extent of the sun for casting soft shadows.
    /// Higher numbers are more diffuse. 5 is a good starting value. Remember: on cloudy days,
    /// the shadows will be blurred because the sunlight is being diffused by clouds.
    /// Try 90 for hazy days, and use 135 for cloudy days, rain and fog.
    /// If you're trying a night atmosphere, sometimes you'll need up to 180.
    /// You'll also have to turn down the shadow alpha in your shadow_control entity -
    /// try changing the Shadow Color variable to something such as 50 50 50.
    pub sun_spread_angle: f64,
    // Specular Color here, only for insurgency and day of defeat.
}

// impl ToLower<Entity<String, String>> for LightEnviroment {
//     fn to_lower(&self) -> Entity<String, String> {}
// }

impl<S: Default> Default for LightEnviroment<S> {
    fn default() -> Self {
        Self {
            point_entity: PointEntity::default(),
            pitch: Default::default(),
            direct_color: ColorBrightness::new(255, 255, 255, 200),
            amb_color: ColorBrightness::new(255, 255, 255, 200),
            direct_color_hdr: None,
            direct_hdr_scale: 1.0, // scales are is 0.7 by default tested in l4d2 hammer
            amb_color_hdr: None,
            amb_hdr_scale: 1.0,
            sun_spread_angle: 5.0, // wiki says this is a good value
        }
    }
}

impl<'a, S> ToLower<Entity<S>> for LightEnviroment<S>
where
    S: AsRef<str> + Clone + From<&'a str> + From<String>,
{
    fn into_lower(self) -> Entity<S> {
        let pitch = self.pitch.unwrap_or(-self.point_entity.angles.pitch);
        let direct_color_hdr = self.direct_color_hdr.unwrap_or(self.direct_color.clone());
        let amb_color_hdr = self.amb_color_hdr.unwrap_or(self.amb_color.clone());

        let mut entity = self.point_entity.into_lower();
        let props = &mut entity.props;
        props.reserve_exact(8);

        props.push(Property::new("pitch", pitch.to_string()));
        props.push(Property::new("_light", self.direct_color.to_string()));
        props.push(Property::new("_ambient", self.amb_color.to_string()));
        props.push(Property::new("_lightHDR", direct_color_hdr.to_string()));
        props.push(Property::new("_lightscaleHDR", self.direct_hdr_scale.to_string()));
        props.push(Property::new("_ambientHDR", amb_color_hdr.to_string()));
        props.push(Property::new("_AmbientScaleHDR", self.amb_hdr_scale.to_string()));
        props.push(Property::new("SunSpreadAngle", self.sun_spread_angle.to_string()));

        entity
    }
}

// TODO:DOCS: todo include c fields
// TODO: is angles needed? not on wiki be is in hammer and there is dir/angle input IO
/// See also: <https://developer.valvesoftware.com/wiki/Shadow_control>
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowControl<S> {
    /// Common Fields. Including classname, origin, angles, flags, etc.
    pub point_entity: PointEntity<S>,

    /// This is the color of the shadows.
    pub shadow_color: RGB8,
    /// This is the maximum distance the shadow is allowed to cast, in inches.
    pub max_dist: f64,
    /// Disable shadows entirely.
    pub disabled: bool,
    // NOTE:GAME: l4d2+, gmod, others
    /// Enable shadow direction to be calculated on a per-entity basis and to be dictated by the light closest to the entity.
    pub local_light_shadows: bool,
}

impl<S: Default> Default for ShadowControl<S> {
    fn default() -> Self {
        Self {
            point_entity: PointEntity::default(),
            shadow_color: RGB8::new(128, 128, 128),
            max_dist: 75.0,
            disabled: false,
            local_light_shadows: true,
        }
    }
}

impl<'a, S> ToLower<Entity<S>> for ShadowControl<S>
where
    S: AsRef<str> + Clone + From<&'a str> + From<String>,
{
    fn into_lower(self) -> Entity<S> {
        let mut entity = self.point_entity.into_lower();
        let props = &mut entity.props;
        props.reserve_exact(4);

        props.push(Property::new("color", self.shadow_color.to_string()));
        props.push(Property::new("distance", self.max_dist.to_string()));
        props.push(Property::new("disableallshadows", self.disabled.to_string()));
        props.push(Property::new(
            "enableshadowsfromlocallights",
            (self.local_light_shadows as i32).to_string(),
        ));

        entity
    }
}

// TODO:DOCS:
/// See also: <https://developer.valvesoftware.com/wiki/Env_fog_controller>
#[derive(Clone, Debug, PartialEq)]
pub struct EnvFogController<S> {
    /// Common Fields. Including classname, origin, angles, flags, etc.
    pub point_entity: PointEntity<S>,
    /// Flag. If multiple env_fog_controllers are active, this one will always take priority. There must be at lease one fog_volume in the map to work!
    pub is_master: bool,

    /// Fog Enable `(fogenable)` Make fog start active.
    pub start_enabled: bool,
    /// Fog Start `(fogstart)` How far away from the viewer the fog should start.
    pub start: f64,
    /// Fog End `(fogend)` How far away from the viewer the fog reaches Fog Max Density.
    pub end: f64,
    /// Fog Max Density `(fogmaxdensity)` Maximum density the fog may reach. Expressed as a decimal percent, so for 45% put 0.45.
    pub max_density: f64,
    /// Far Z Clip Plane `(farz)` Anything beyond this distance in world units will not be rendered.
    /// This should be higher than Fog End. If this is used, Fog Max Density should be set to 1
    /// otherwise the void may be visible.
    ///
    /// -1 means none.
    pub far_z: i32,
    /// Primary Fog Color `(fogcolor)` Primary Fog Color.
    pub primary_color: RGB8,
    /// Secondary Fog Color `(fogcolor2)` Secondary Fog Color. If Fog Blend is disabled, this color will never appear.
    pub secondary_color: RGB8,
    /// Fog Blend `(fogblend)` Enables color blending between Primary Fog Color and Secondary Fog Color. When the viewer looks in the Primary Fog Direction, fog will appear as the Primary color. When looking away from the specified direction, fog appears as the Secondary color. If the camera is not pointed directly at or away from the direction, a blend of the two colors will result. Sunlight with a Yaw of 45 degrees and a Pitch of -45 degrees could be enhanced using a Primary Fog Direction of "-1 -1 1", a Primary Fog Color of "120 110 100" and a Secondary Fog Color of "80 70 60".
    pub use_blend: bool,
    /// Primary Fog Direction `(fogdir)` A vector (given by three space-separated numbers X Y Z) which the viewer camera is checked against to figure out the blend between the primary and secondary fog colors.
    pub primary_blend_dir: Vector3<f64>,
    /// Use Angles for Fog Dir `(use_angles)` Use Pitch Yaw Roll for the Fog Blend direction instead of Primary Fog Direction. There isn't much use for this unless you want the direction to rotate.
    pub use_angles_for_dir: bool,
    /// Interpolate time `(foglerptime)` Fade time for the StartFogTransition input.
    pub interp_time: f64,
    // l4d2+
    /// HDR Color Scale `(HDRColorScale)` Multiplier for fog color when in HDR mode.
    pub hdr_color_scale: f64,
    // Zoom Fog Scale `(ZoomFogScale)` Scalar for fog start and end distances when the player is looking through a gun's scope (e.g. AWP).
}

// TODO: use angles and sun angles instead
impl<S: Default> Default for EnvFogController<S> {
    fn default() -> Self {
        Self {
            point_entity: PointEntity::default(),
            is_master: false,

            start_enabled: true,
            start: 500.0,
            end: 2000.0,
            max_density: 1.0,
            far_z: 2100,
            primary_color: RGB8::new(255, 255, 255),
            secondary_color: RGB8::new(255, 255, 255),
            use_blend: false,
            primary_blend_dir: Vector3::new(1.0, 0.0, 0.0),
            use_angles_for_dir: false,
            interp_time: 1.0,
            hdr_color_scale: 1.0,
        }
    }
}

impl<'a, S> ToLower<Entity<S>> for EnvFogController<S>
where
    S: AsRef<str> + Clone + From<&'a str> + From<String>,
{
    fn into_lower(mut self) -> Entity<S> {
        self.point_entity.flags = self.is_master as i32;
        let mut entity = self.point_entity.into_lower();
        let props = &mut entity.props;
        props.reserve_exact(11);

        props.push(Property::new("fogenable", (self.start_enabled as i32).to_string()));
        props.push(Property::new("fogstart", self.start.to_string()));
        props.push(Property::new("fogend", self.end.to_string()));
        props.push(Property::new("fogmaxdensity", self.max_density.to_string()));
        props.push(Property::new("farz", self.far_z.to_string()));
        props.push(Property::new("fogcolor", self.primary_color.to_string()));
        props.push(Property::new("fogcolor2", self.secondary_color.to_string()));
        props.push(Property::new("fogblend", (self.use_blend as i32).to_string()));
        props.push(Property::new("fogdir", self.primary_blend_dir.to_string()));
        props.push(Property::new("use_angles", (self.use_angles_for_dir as i32).to_string()));
        props.push(Property::new("foglerptime", self.interp_time.to_string()));
        props.push(Property::new("HDRColorScale", self.hdr_color_scale.to_string()));

        entity
    }
}
