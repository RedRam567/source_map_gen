//! CIE 1931 XYZ, CIE 1931 xy, and Delta uv

use super::SrgbLinear;
use rgb::RGB;

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
        pub(crate) const K: [f64; 7] =
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
pub(crate) fn xy_to_cct_duv(xy: Xy) -> (f64, f64) {
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
