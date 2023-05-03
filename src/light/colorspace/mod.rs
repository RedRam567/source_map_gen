#![allow(non_snake_case)] // try to avoid some confusion between XYZ and xy
//! CIE and sRGB colorspaces and conversions.

mod cie;

pub use cie::*;

use rgb::RGB;
use std::ops::{Deref, DerefMut};

//https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf
// TODO: clean

/// Linear sRGB in the range 0 to 1
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct SrgbLinear(pub RGB<f64>);

impl SrgbLinear {
    // TODO: negatives, add until pos then normalize
    // or clamp to zero
    /// Normalizes `self` into the range [0, 1].
    /// Does not handle negatives.
    pub fn normalize_mut(&mut self) -> &mut Self {
        let RGB { r, g, b } = self.0;
        let max = (r.max(g)).max(b);
        if max == 0.0 {
            // avoid divide by zero
            *self = SrgbLinear::default();
            return self;
        }
        self.r = r / max;
        self.g = g / max;
        self.b = b / max;
        self
    }

    /// Convert from linear sRGB in the range 0 to 255 into linear sRGB in the range 0 to 1.
    pub fn from_linear(rgb: RGB<u8>) -> Self {
        let rgb = RGB { r: rgb.r as f64 / 255.0, g: rgb.g as f64 / 255.0, b: rgb.b as f64 / 255.0 };
        Self(rgb)
    }

    // /// Convert from sRGB in the range 0 to 255 into linear sRGB in the range 0 to 1.
    // pub fn from_srgb(srgb: RGB<u8>) -> Self {
    //     let RGB { r, g, b } = srgb;

    //     let rgb = RGB {
    //         r: (r as f64 / 255.0).powf(2.2),
    //         g: (g as f64 / 255.0).powf(2.2),
    //         b: (b as f64 / 255.0).powf(2.2),
    //     };
    //     Self(rgb)
    // }

    // /// To normal sRGB in range 0 to 255.
    // /// TODO: verify
    // pub fn to_srgb(&self) -> RGB<u8> {
    //     /// 1.0 / 2.2
    //     const INVERSE_GAMMA: f64 = 0.45454545454545454;
    //     let r = (self.r.powf(INVERSE_GAMMA) * 255.0) as u8;
    //     let g = (self.g.powf(INVERSE_GAMMA) * 255.0) as u8;
    //     let b = (self.b.powf(INVERSE_GAMMA) * 255.0) as u8;
    //     RGB { r, g, b }
    // }

    /// Convert from sRGB in the range 0 to 255 into linear sRGB in the range 0 to 1.
    pub fn to_srgb(&self) -> RGB<u8> {
        // let RGB { r, g, b } = self;

        // let rgb = RGB {
        //     r: (r as f64 / 255.0).powf(2.2),
        //     g: (g as f64 / 255.0).powf(2.2),
        //     b: (b as f64 / 255.0).powf(2.2),
        // };
        // Self(rgb)
        // eprintln!("{:?}", self);
        const INVERSE_GAMMA: f64 = 0.45454545454545454;

        let r = ((self.r).powf(INVERSE_GAMMA) * 255.0) as u8;
        let g = ((self.g).powf(INVERSE_GAMMA) * 255.0) as u8;
        let b = ((self.b).powf(INVERSE_GAMMA) * 255.0) as u8;
        RGB { r, g, b }
    }

    /// To normal sRGB in range 0 to 255.
    /// TODO: verify
    pub fn from_srgb(srgb: &RGB<u8>) -> Self {
        /// 1.0 / 2.2
        // const INVERSE_GAMMA: f64 = 0.45454545454545454;
        // const INVERSE_GAMMA: f64 = 0.45454545454545454;
        const GAMMA: f64 = 2.2;
        let r = (srgb.r as f64 / 255.0).powf(GAMMA);
        let g = (srgb.g as f64 / 255.0).powf(GAMMA);
        let b = (srgb.b as f64 / 255.0).powf(GAMMA);
        SrgbLinear(RGB { r, g, b })
    }

    /// Convert to CIE XYZ. See [`Xyz`]
    ///
    /// More info:
    /// <http://www.brucelindbloom.com/index.html?Eqn_T_to_xy.html>,
    /// <https://en.wikipedia.org/wiki/CIE_1931_color_space>.
    pub fn to_xyz(&self) -> Xyz {
        let RGB { r, g, b } = self.0;
        // fn c_to_linear(c: f64) -> f64 {
        //     if c <= 0.04045 {
        //         c / 12.92
        //     } else {
        //         ((c + 0.055) / 1.055).powf(2.4)
        //     }
        // }
        // let r = c_to_linear(r);
        // let g = c_to_linear(g);
        // let b = c_to_linear(b);
        let r = r * 100.0;
        let g = g * 100.0;
        let b = b * 100.0;
        let X = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
        let Y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
        let Z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
        Xyz { X, Y, Z }
    }
}

impl Deref for SrgbLinear {
    type Target = RGB<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SrgbLinear {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<RGB<f64>> for SrgbLinear {
    fn as_ref(&self) -> &RGB<f64> {
        &self.0
    }
}

impl AsMut<RGB<f64>> for SrgbLinear {
    fn as_mut(&mut self) -> &mut RGB<f64> {
        &mut self.0
    }
}

/// Closely approximates the RBG color of a color temperature in Kelvin.
/// Not scientifically rigorous but pretty close with an R-squared value of ~0.988.
///
/// Adapted from <https://tannerhelland.com/2012/09/18/convert-temperature-rgb-algorithm-code.html>.
fn temp_to_rgb_simple(kelvin: f64) -> (u8, u8, u8) {
    let temp = kelvin / 100.0;

    let red;
    let green;
    let blue;

    // ifs merged to improve codegen (rip readability)
    // clamps unneeded, f64 as u8 clamps already
    // as cast: NaN or -NaN -> 0, infinites to closest
    if temp <= 66.0 {
        red = 255;

        let g = temp.ln();
        let g = 99.4708025861 * g - 161.1195681661;
        green = g as u8;

        blue = 255;
    } else {
        let r = temp - 60.0;
        let r = 329.698727446 * r.powf(-0.1332047592);
        red = r as u8;

        let g = temp - 60.0;
        let g = 288.1221695283 * g.powf(-0.0755148492);
        green = g as u8;

        if temp <= 19.0 {
            blue = 0;
        } else {
            let b = temp - 10.0;
            let b = 138.5177312231 * b.ln() - 305.0447927307;
            blue = b as u8;
        }
    };

    (red, green, blue)
}

// /// Convert from correlated color temperature (CCT) to CIE 1931 xy.
// /// Accurate from ~4,000K to ~25,000K
// /// Adapted from <http://www.brucelindbloom.com/index.html?Eqn_T_to_xy.html>.
// pub fn cct_to_xy(cct: f64) -> Xy {
//     let cct = cct;
//     let cct_2 = cct * cct;
//     let cct_3 = cct_2 * cct;
//     let ten_3_t = 10e3 / cct;
//     let ten_6_t_2 = 10e6 / cct_2;
//     let ten_9_t_3 = 10e9 / cct_3;
//     let x = if cct <= 7000.0 {
//         // accurate down to 4000k
//         -4.6070 * ten_9_t_3 + 2.9678 * ten_6_t_2 + 0.9911 * ten_3_t + 0.244063
//     } else {
//         // accurate up to 25000k
//         dbg!("e");
//         -2.0064 * ten_9_t_3 + 1.9018 * ten_6_t_2 + 0.24748 * ten_3_t + 0.237040
//     };
//     let x = x / 10.0; // why is this nessessary??
//     let y = -3.000 * x * x + 2.870 * x - 0.275;
//     // dbg!(x,y, cct);
//     Xy { x, y }
// }

// WRONG
// pub fn cct_to_xy(cct: f64) -> Xy {
//     let t = cct;
//     let t_2 = t * t;
//     let t_3 = t_2 * t;
//     let ten_9_t_3 = 10e9 / t_3;
//     let ten_6_t_2 = 10e6 / t_2;
//     let ten_3_t = 10e3 / t;

//     let x = if t <= 4000.0 {
//         // accurate down to 1667K
//         // assert!(t >= 1667.0);
//         // assert!(t <= 4000.0);
//         eprint!("x1 ");
//         ( -0.2661239 * ten_9_t_3 - 0.2343589 * ten_6_t_2 + 0.8776956 * ten_3_t + 0.179910 )
//     } else {
//         // accurate up to 25000k
//         // assert!(t >= 4000.0);
//         // assert!(t <= 25000.0);
//         eprint!("x2 ");
//         ( -3.0258469 * ten_9_t_3 + 2.1070379 * ten_6_t_2 + 0.2226347 * ten_3_t + 0.240390 )
//     };

//     let x_2 = x * x;
//     let x_3 = x_2 * x;

//     let y = if t <= 2222.0 {
//         // accurate down to 1667K
//         assert!(t >= 1667.0);
//         assert!(t <= 2222.0);
//         eprintln!("y1");
//         -1.1063814 * x_3 - 1.34811020 * x_2 + 2.18555832 * x - 0.20219683
//     } else if t <= 4000.0 {
//         assert!(t >= 2222.0);
//         assert!(t <= 4000.0);
//         // accurate
//         eprintln!("y2");
//         -0.9549476 * x_3 - 1.37418593 * x_2 + 2.09137015 * x - 0.16748867
//     } else {
//         assert!(t >= 4000.0);
//         // assert!(t <= 25000.0);
//         // accurate up to 25000K
//         eprintln!("y3");
//         3.0817580 * x_3 - 5.87338670 * x_2 + 3.75112997 * x - 0.37001483
//     };

//     if x > 0.75 || y > 0.8 {
//         eprintln!("imaginary");
//     }

//     dbg!(x,y);

//     Xy { x, y }
// }

/// Convert sRGB to corelated color temperature (cct).
/// Input is your normal, gamma corrected (not linear) sRGB values.
pub fn rgb_to_temp(srgb: RGB<u8>) -> f64 {
    // let rgb = Rgb { r: todo!(), g: todo!(), b: todo!() };
    // let mut rgb = SrgbLinear::from_linear_u8(rgb);

    if srgb.r == 230 {
        dbg!(&srgb);
    }
    let rgb = SrgbLinear::from_srgb(&srgb);
    if srgb.r == 230 {
        dbg!(&rgb);
    }

    // let input = srgb;
    // let rgb: SrgbLinear = SrgbLinear::from_linear(srgb);
    // let mut rgb = srgb_to_linear(rgb);
    // rgb.normalize_mut();
    let XYZ = rgb.to_xyz();
    if srgb.r == 230 {
        dbg!(&XYZ);
    }
    let (xy, _) = XYZ.to_xyy();
    if srgb.r == 230 {
        dbg!(&xy);
    }

    let (cct, _) = xy_to_cct_duv(xy);
    if srgb.r == 230 {
        dbg!(&cct);
    }
    cct
}

pub fn temp_to_rgb(cct: f64) -> RGB<u8> {
    if cct as i32 == 3805 {
        eprintln!("e\n");
        let xy = dbg!(cct_to_xy(cct));
        let xyz = dbg!(xy.to_xyz(50.0));
        let rgb = dbg!(xyz.to_rgb());
    }
    let xy = cct_to_xy(cct);
    let xyz = xy.to_xyz(100.0);
    let mut rgb = xyz.to_rgb();
    // dbg!(&xy);
    rgb.normalize_mut();
    rgb.to_srgb()
}

#[cfg(test)]
mod tests {
    use approx::abs_diff_eq;

    use super::*;

    #[test]
    fn test() {
        let input = RGB::new(230, 186, 138);
        dbg!(&input);
        // let rgb_linear: SrgbLinear = SrgbLinear::from_linear(input);
        // let rgb_linear = srgb_to_linear(rgb.clone());
        let rgb = SrgbLinear::from_srgb(&input);
        let XYZ = rgb.to_xyz();

        // let XYZ = rgb_linear.to_xyz();
        // let rgb = rgb_linear;
        dbg!(&XYZ);
        assert!(abs_diff_eq!(54.7, XYZ.X, epsilon = 1.0));
        assert!(abs_diff_eq!(53.7, XYZ.Y, epsilon = 1.0));
        assert!(abs_diff_eq!(31.5, XYZ.Z, epsilon = 1.0));

        let (xy, Y) = XYZ.to_xyy();
        assert!(abs_diff_eq!(0.391, xy.x, epsilon = 0.01));
        assert!(abs_diff_eq!(0.384, xy.y, epsilon = 0.01));
        assert_eq!(XYZ.Y, Y);

        // let xy = Xy { x: xy.x / 1.0, y: xy.y / 1.0 };
        dbg!(&xy);

        let (cct, d_uv) = xy_to_cct_duv(xy.clone());
        dbg!(cct);
        // small d_uv = almost perfectly on black body line
        assert!(abs_diff_eq!(0.0, d_uv, epsilon = 0.01));
        assert!(abs_diff_eq!(3800.0, cct, epsilon = 50.0));

        // now back to rgb

        let new_xy = cct_to_xy(cct);
        dbg!(&new_xy);
        assert!(abs_diff_eq!(xy.x, new_xy.x, epsilon = 0.05));
        assert!(abs_diff_eq!(xy.y, new_xy.y, epsilon = 0.05));

        // Y is luminance, doesnt matter as its normalized later
        // just do 100 for max I THINK
        let new_XYZ = new_xy.to_xyz(100.0);
        dbg!(&new_XYZ);

        let mut new_rgb = new_XYZ.to_rgb();
        new_rgb.normalize_mut();
        let new_rgb = new_rgb.to_srgb();
        dbg!(&new_rgb);
        assert!(abs_diff_eq!(255, new_rgb.r, epsilon = 0));
        assert!(abs_diff_eq!(205, new_rgb.g, epsilon = 0));
        assert!(abs_diff_eq!(154, new_rgb.b, epsilon = 0));

        // back to rgb, scaled brightness

        let mut new_rgb2 = new_XYZ.to_rgb();
        new_rgb2.normalize_mut();

        let max = rgb.r.max(rgb.g).max(rgb.b);
        new_rgb2.r *= max;
        new_rgb2.g *= max;
        new_rgb2.b *= max;

        let new_rgb2 = new_rgb2.to_srgb();
        dbg!(&new_rgb2);
        // kinda the same, good enough, prob better to normalize xyY or XYZ or
        // smth idfk at this point
        assert!(abs_diff_eq!(input.r, new_rgb2.r, epsilon = 20));
        assert!(abs_diff_eq!(input.g, new_rgb2.g, epsilon = 20));
        assert!(abs_diff_eq!(input.b, new_rgb2.b, epsilon = 20));
    }

    // fn
}
