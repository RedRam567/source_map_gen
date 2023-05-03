//https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf

use std::ops::{Deref, DerefMut};

use rgb::RGB;

/// Linear sRGB in the range 0 to 1
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct SrgbLinear(pub RGB<f64>);

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

/// CIE 1931 XYZ color space. Not to be confused with CIE 1931 xy chromaticity
/// See [`Xy`]
///
/// More info: <https://en.wikipedia.org/wiki/CIE_1931_color_space>.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Xyz {
    /// `X` in the range 0 to 95.047.
    pub X: f64,
    /// `Y` (lumenosicty) in the range 0 to 100.
    pub Y: f64,
    /// `Z` in the range 0 to 108.883.
    pub Z: f64,
}

/// CIE 1931 xy chromaticity. Not to be confused with CIE 1931 XYZ color space.
/// See [`Xyz`]
///
/// More info: <https://en.wikipedia.org/wiki/CIE_1931_color_space>.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Xy {
    /// `x` in the range 0 to ~0.75
    pub x: f64,
    /// `y` in the range 0 to ~0.85
    pub y: f64,
}

// TODO: verify
/// CIE 1960 uv and delta uv. Maybe CIE 1931 or CIE 1976, I don't know.
/// More info: <https://en.wikipedia.org/wiki/CIE_1931_color_space>,
/// <https://en.wikipedia.org/wiki/CIE_1960_color_space>,
/// <https://en.wikipedia.org/wiki/CIELUV>.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Duv {
    // not sure what this is
    pub u: f64,
    // not sure what this is
    pub v: f64,
    /// An amount "perpendicular" to the Plankian Locus
    /// with identical ccts (correlated color temperature).
    /// Positve is "above", towardish positive `y` or greenish/yellowish.
    /// Negative is "below", towardish negative `y` or megentaish.
    pub d_uv: f64,
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

impl Xyz {
    // TODO: normalize, etc, where did /100 go??
    /// Convert to [`SrgbLinear`]
    ///
    /// More info:
    /// <http://www.brucelindbloom.com/index.html?Eqn_T_to_xy.html>,
    /// <https://en.wikipedia.org/wiki/CIE_1931_color_space>.
    pub fn to_rgb(&self) -> SrgbLinear {
        let Xyz { X, Y, Z } = *self;
        let r = 3.2404542 * X + -1.5371385 * Y + -0.4985314 * Z;
        let g = -0.9692660 * X + 1.8760108 * Y + 0.0415560 * Z;
        let b = 0.0556434 * X + -0.2040259 * Y + 1.0572252 * Z;
        let r = r / 255.0;
        let g = g / 255.0;
        let b = b / 255.0;
        // dbg!(r,g,b);
        SrgbLinear(RGB { r, g, b })
    }

    /// CIE XYZ to CIE xyY.
    /// See <https://en.wikipedia.org/wiki/CIE_1931_color_space>.
    pub fn to_xyy(&self) -> (Xy, f64) {
        let Xyz { X, Y, Z } = *self;
        let x = X / (X + Y + Z);
        let y = Y / (X + Y + Z);
        // wtf is z??
        // let z = 1.0 - x - y

        (Xy { x, y }, Y)
    }
}

impl Xy {
    /// CIE xyY to CIE XYZ.
    /// See <https://en.wikipedia.org/wiki/CIE_1931_color_space>.
    pub fn to_xyz(&self, Y: f64) -> Xyz {
        let Xy { x, y } = *self;
        let X = Y / y * x;
        let Z = Y / y * (1.0 - x - y);
        Xyz { X, Y, Z }
    }

    /// Convert to [`Duv`].
    /// Different calculation than the `Duv` in TODO: xyz_to_cct_duv
    ///
    /// # More Info
    /// <https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf>.
    pub fn to_duv(&self) -> Duv {
        const K: [f64; 7] =
            [-0.471106, 1.925865, -2.4243787, 1.5317403, -0.5179722, 0.0893944, -0.00616793];

        let x = self.x;
        let y = self.y;

        let div = -2.0 * x + 12.0 * y + 3.0;
        let u = 4.0 * x / div;
        let v = 6.0 * y / div;

        // hypot is sqrt(a^2 + b^2)
        let l_fp = (u - 0.292).hypot(v - 0.24);
        let a = ((u - 0.292) / l_fp).acos();
        let l_bb = K[6] * a.powi(6)
            + K[5] * a.powi(5)
            + K[4] * a.powi(4)
            + K[3] * a.powi(3)
            + K[2] * a.powi(2)
            + K[1] * a
            + K[0];
        let d_uv = l_fp - l_bb;

        Duv { d_uv, u, v }
    }
}

// TODO: colorspace mod, tests tests tests
// https://www.easyrgb.com/en/convert.php#inputFORM
//https://en.wikipedia.org/wiki/CIE_1931_color_space
//https://github.com/tompazourek/Colourful/blob/11401fea462505317669fc509d9618344344cb9e/src/Colourful/Utils/CCTConverter.cs
//https://web.archive.org/web/20190303161843/http://pdfs.semanticscholar.org/cc7f/c2e67601ccb1a8fec048c9b78a4224c34d26.pdf
//https://en.wikipedia.org/wiki/Planckian_locus#Approximation
//https://en.wikipedia.org/wiki/Standard_illuminant
//https://stackoverflow.com/questions/61262783/convert-cct-correlated-color-temperature-to-x-y-chromaticities
//https://tannerhelland.com/2012/09/18/convert-temperature-rgb-algorithm-code.html

//http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
//https://en.wikipedia.org/wiki/SRGB

//https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf#18

//https://andi-siess.de/rgb-to-color-temperature/

//https://en.wikipedia.org/wiki/CIE_1931_color_space
//https://en.wikipedia.org/wiki/CIE_1960_color_space
//https://observablehq.com/@bokub/xy-to-rgb
//https://www.waveformlighting.com/files/blackBodyLocus_1.txt

//https://www.waveformlighting.com/tech/calculate-cie-1931-xy-coordinates-from-cct/
//http://dougkerr.net/Pumpkin/articles/CIE_XYZ.pdf
//http://colormine.org/convert/rgb-to-xyz

//https://github.com/colour-science/colour/blob/develop/colour/temperature/cie_d.py

//https://en.wikipedia.org/wiki/Correlated_color_temperature#Approximation
//https://en.wikipedia.org/wiki/Planckian_locus#cite_note-3
//https://www.semanticscholar.org/paper/Design-of-advanced-color%3A-Temperature-control-for-Kang-Moon/cc7fc2e67601ccb1a8fec048c9b78a4224c34d26

// // TODO: verify when needed
// /// Gamma-expand sRGB to linear RGB. Must be in the range `[0, 1]`
// ///
// /// See also <https://en.wikipedia.org/wiki/SRGB>.
// ///
// /// # Notes
// /// I'm not sure if/when this is nessessary, for example, when getting color
// /// temperature of a pixel from an image.
// pub fn srgb_to_linear(srgb: SrgbLinear) -> SrgbLinear {
//     fn c_to_linear(c: f64) -> f64 {
//         if c <= 0.04045 {
//             c / 12.92
//         } else {
//             ((c + 0.055) / 1.055).powf(2.4)
//         }
//     }
//     let r = c_to_linear(srgb.r);
//     let g = c_to_linear(srgb.g);
//     let b = c_to_linear(srgb.b);
//     SrgbLinear(RGB { r, g, b })
// }

/// Get correlated color temperature (CCT) and D<sub>uv</sub> from a CIE XYZ color.
/// CCT is basically the closet color temperature to the ideal color temperature curve.
/// D<sub>uv</sub> is basically how green (positve) or magenta (negative) the color is.
/// The D<sub>uv</sub> this returns is different than [`Xy::to_duv()`]
///
/// See also: [`Xy`] and [`Duv`].
///
/// # More Info
/// * <https://en.wikipedia.org/wiki/CIE_1931_color_space>
///
/// Adapted from <https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf>.
fn xy_to_cct_duv(xy: Xy) -> (f64, f64) {
    // xclip -o -sel clip | sed -e ':a;N;$!ba; s/0\n/0/g; s/8\n6/8 6/g' | awk -v OFS=', ' '{print "[" $8, $7, $6, $5, $4, $3, $2 ", ]"}' | column -t

    // magic table
    // k_16 -> K[1][6]
    #[rustfmt::skip]
    const K: [[f64; 7]; 7] = [
        [-1.77348E-01,     1.115559E+00,      -1.5008606E+00,     9.750013E-01,       -3.307009E-01,       5.6061400E-02,    -3.7146000E-03,  ],
        [5.308409E-04,     2.1595434E-03,     -4.3534788E-03,     3.6196568E-03,      -1.589747E-03,       3.5700160E-04,    -3.2325500E-05,  ],
        [-8.58308927E-01,  1.964980251E+00,   -1.873907584E+00,   9.53570888E-01,     -2.73172022E-01,     4.17781315E-02,   -2.6653835E-03,  ],
        [-2.3275027E+02,   1.49284136E+03,    -2.7966888E+03,     2.51170136E+03,     -1.1785121E+03,      2.7183365E+02,    -2.3524950E+01,  ],
        [-5.926850606E+08, 1.34488160614E+09, -1.27141290956E+09, 6.40976356945E+08,  -1.81749963507E+08,  2.7482732935E+07, -1.731364909E+06,],
        [-2.3758158E+06,   3.89561742E+06,    -2.65299138E+06,    9.60532935E+05,     -1.9500061E+05,      2.10468274E+04,   -9.4353083E+02,  ],
        [2.8151771E+06,    -4.11436958E+06,   2.48526954E+06,     -7.93406005E+05,    1.4101538E+05,       -1.321007E+04,    5.0857956E+02,   ],
    ];

    // TODO: optimize this into and merge with pows() to remove intermediate array
    // (is that even better?)
    #[inline]
    #[rustfmt::skip]
    fn pow_sum(k: &[f64; 7], a: &[f64; 6]) -> f64 {
        k[6] * a[5]
            + k[5] * a[4]
            + k[4] * a[3]
            + k[3] * a[2]
            + k[2] * a[1]
            + k[1] * a[0]
            + k[0]
    }

    /// a^1, a^2, ... a^6
    #[inline(always)]
    fn pows(a: f64) -> [f64; 6] {
        let a_2 = a * a;
        let a_4 = a_2 * a_2;
        [a, a_2, a_2 * a, a_4, a_4 * a, a_4 * a_2]
    }

    // Calculate u and v
    // NOTE: pretty im supossed to use the impl in xy::to_duv() for u and v
    // but use this fns l_bb and d_uv
    let x = xy.x;
    let y = xy.y;
    let div = -2.0 * x + 12.0 * y + 3.0;
    let u = 4.0 * x / div;
    let v = 6.0 * y / div;

    // hypot is sqrt(a^2 + b^2)
    let l_fp = (u - 0.292).hypot(v - 0.24);
    let a_1 = ((v - 0.24) / (u - 0.292)).atan();
    let a = if a_1 >= 0.0 { a_1 } else { a_1 + std::f64::consts::PI };

    let a_pows = pows(a);
    let l_bb = pow_sum(&K[0], &a_pows);
    let d_uv = l_fp - l_bb;

    // Calculate t_1's
    let t_1;
    let dt_c1;
    // NOTE: im ASSUMING l_p is a typo and the paper meant l_fp
    let mul = (l_bb + 0.01) / l_fp * d_uv / 0.01;
    if a < 2.54 {
        t_1 = pow_sum(&K[1], &a_pows).recip();
        dt_c1 = pow_sum(&K[3], &a_pows) * mul;
    } else {
        t_1 = pow_sum(&K[2], &a_pows).recip();
        dt_c1 = pow_sum(&K[4], &a_pows).recip() * mul;
    };

    // Calculate t_2's
    let t_2 = t_1 - dt_c1;
    let c = t_2.log10(); // wtf log is log10 not ln in scientific paper
    let c_pows = pows(c);

    let dt_c2 = if d_uv >= 0.0 {
        pow_sum(&K[5], &c_pows)
    } else {
        pow_sum(&K[6], &c_pows) * (d_uv / 0.03).abs().powi(2)
    };

    let t_final = t_2 - dt_c2;

    (t_final, d_uv)
}

/// Approximates [CIE 1931 xy][xy] coordanites from a [CCT][cct].
/// Other formulas were giving bogus results so I made my own.
/// Approximates the [data table][table] from the calculator at [www.waveformlighting.com][calc].
/// Accurate to about 3.5 digit places. Maximum total error of about 4.7e-4.
///
/// [xy]: https://en.wikipedia.org/wiki/CIE_1931_color_space#CIE_xy_chromaticity_diagram_and_the_CIE_xyY_color_space
/// [cct]: https://en.wikipedia.org/wiki/Correlated_color_temperature
/// [table]: https://www.waveformlighting.com/files/blackBodyLocus_1.txt
/// [calc]: https://www.waveformlighting.com/tech/calculate-cie-1931-xy-coordinates-from-cct
#[rustfmt::skip]
pub fn cct_to_xy(cct: f64) -> Xy {
    // trained on recip of cct as it was more accurate.
    // 100 / t or 1000 / t or smth made nicer coeffs but same error
    let t = cct.recip();
    let t_2 = t * t;
    let t_3 = t_2 * t;

    let x = if cct <= 1667.0 {
        // error of -9.84877e-5 at 1000, untested below
        // error of -7.07369e-5 at 1666
        2.462938165625e8 * t_3    - 8.243476920166016e5  * t_2 + 1.0554542694687843e3 * t + 1.7544814945722464e-1 // for -inf < x <= 1667
    } else if cct <= 4000.0 {
        // error of  4.75410e-4 at 1668
        // error of  3.18018e-4 at 3999
        -9.0977539265625e7 * t_3  - 4.7475309005737305e5 * t_2 + 9.8270661521703e2 * t    + 1.6553588082842907e-1 // for 1667 < x <= 4000
    } else {
        // error of -6.01152e-7 at 4001
        // error of -7.66828e-6 at 10_000
        // error of -2.50238e-5 at 20_000, ok above (no data to verify)
        -3.29527249421875e9 * t_3 + 2.228834467737198e6  * t_2 + 2.0652059505344369e2 * t + 2.409952563527895e-1  // for 4000 < x <= inf
    };
    
    let y = if cct <= 1667.0 {
        // error of -1.60716e-4 at 1000, untested below
        // error of -1.35235e-4 at 1666
        1.91721760625e8 * t_3     - 4.302777307739258e5 * t_2  + 1.6706878939270973e2 * t + 4.1611012386420043e-1 // for -inf < x <= 1667
    } else if cct <= 2222.0 {
        // error of -1.05763e-5 at 1668
        // error of -7.92373e-6 at 2221
        1.144798462e9 * t_3       - 2.2162549921875e6 * t_2    + 1.2905086498260498e3 * t + 1.7900749668478966e-1 // for 1667 < x <= 2222
    } else if cct <= 4000.0 {
        // error of  5.64593e-5 at 2223
        // error of  5.13267e-5 at 3999
        1.207743859e9 * t_3       - 2.4118299509277344e6 * t_2 + 1.4248815863132477e3 * t + 1.5234232398142922e-1 // for 2222 < x <= 4000
    } else {
        // error of  4.56613e-4 at 4001
        // error of -9.50785e-6 at 10_000
        // error of  8.85447e-5 at 20_000, ok above (no data to verify)
        -5.4520836120625e9 * t_3  + 2.03833329580307e6 * t_2   + 4.0477207188354805e2 * t + 2.3288708110055722e-1 // for 4000 < x <= inf
    };

    Xy {x, y}
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
