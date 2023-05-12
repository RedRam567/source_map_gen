pub mod colorspace;
pub mod time;

use rgb::RGB;

use crate::{
    map::{Angles, Entity},
    source::ColorBrightness,
};

// TODO: cloudyness
// TODO: fog and skybox and env_sun

// TODO: merge with lightenv? keep and split into valve stuff
// TODO:DOCS:
/// All information for global lighting
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalLighting {
    pub sun_color: ColorBrightness,
    pub sun_dir: Angles,
    pub amb_color: ColorBrightness,
    pub amb_dir: Angles, // needed?
    pub dir_lights: Vec<(ColorBrightness, Angles)>,
}

// TODO: thing to entity or entity from??
impl GlobalLighting {
    // light env
    // dir
    // shadow_control https://developer.valvesoftware.com/wiki/Intermediate_Lighting#light
    // color correction
    // env_tonemap_controller (with logic_auto?) // https://developer.valvesoftware.com/wiki/Env_tonemap_controller
    // env_sun (sprite) NEED SKYBOX FIRST TO TARGET
    // lightmap scale
    // csm (realtime shadows) https://developer.valvesoftware.com/wiki/Env_cascade_light
    // csm/shadow_control mutually exclusive?

    // https://developer.valvesoftware.com/wiki/Advanced_Lighting#Compile_Options
    // -staticproppolys // mesh instead of collison mesh
    // -textureshadows // alpha support
    // -staticproplighting // vertex lighting for props
    //     Warning.png Warning: This will disable info_lighting entities on static props that don't use bump maps!
    pub fn to_entity(&self) -> Entity<String> {
        // TODO: here
        todo!()
    }
}

// impl<'a> ToLower<'a, LightEnviroment> for GlobalLighting {

//     fn to_lower(&self) -> LightEnviroment {
//         LightEnviroment {
//             point_entity: PointEntity { angles: self.sun_dir.clone(), ..PointEntity::default() },
//             direct_color: self.sun_color.clone(),
//             amb_color: self.amb_color.clone(),
//             ..LightEnviroment::default()
//         }
//     }

//     fn into_lower(self) -> LightEnviroment {
//         todo!()
//     }
// }

impl Default for GlobalLighting {
    // TODO:
    /// defaults from [`crate::source::LightEnviroment]
    fn default() -> Self {
        Self {
            sun_color: ColorBrightness::new(255, 255, 255, 200),
            sun_dir: Angles::default(),
            amb_color: ColorBrightness::new(255, 255, 255, 50),
            amb_dir: Angles::default(),
            dir_lights: Default::default(),
        }
    }
}

/// Sun pitch to rgb color, negative is night time.
pub fn pitch_to_rgb(pitch: f64) -> RGB<u8> {
    let rgb = if pitch < 0.0 {
        // TODO: add more green or smth
        let mut srgb_linear =
            colorspace::cct_to_xy(night_pitch_to_temp(-pitch)).to_xyz(50.0).to_rgb();
        srgb_linear.normalize_mut();
        srgb_linear.to_srgb()
    } else {
        let mut srgb_linear =
            colorspace::cct_to_xy(day_pitch_to_temp(pitch)).to_xyz(50.0).to_rgb();
        srgb_linear.normalize_mut();
        srgb_linear.to_srgb()
    };
    rgb
}

fn day_pitch_to_temp(pitch: f64) -> f64 {
    debug_assert!(pitch >= 0.0);

    /// arbitrary
    const STEEP_EXP: f64 = 1.0 / 3.0;
    /// Solves: `5000 * 90 ^ EXP = 7000`
    const FLAT_EXP: f64 = 0.07477_47707_7302;

    if pitch < 1.0 {
        // avoid 0 -> 0
        2500.0
    } else if pitch < 14.5969 {
        // intersection point: (14.5969, 6019.8065)
        // a * x ^ b
        2500.0 * pitch.powf(STEEP_EXP)
    } else {
        // a * x ^ b
        5000.0 * pitch.powf(FLAT_EXP)
    }
}

fn night_pitch_to_temp(pitch: f64) -> f64 {
    if pitch < 1.0 {
        2500.0
    } else if pitch < 3.1637 {
        // intersection point: (3.1637, 7909.4095)
        2500.0 * pitch
    } else {
        // solves 5000 * 90 ^ EXP = 30_000
        5000.0 * pitch.powf(0.39818561239203)
    }
}
