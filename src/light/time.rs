use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rgb::RGB;
use spa::{SolarPos, SpaError};

use crate::{
    // light::{pitch_to_rgb, Angles, GlobalLighting},
    source::{ColorBrightness}, light::{GlobalLighting, pitch_to_rgb}, map::Angles,
};

// lat
// long
// localdate
// localtime

// TODO: need:
// r/g/b or temp over time
// actually working temp to rgb
// amb color over time
// brightness overtime

// TODO: solarpos to angles
// TODO: to global light thingy
// TODO: clouds/fire/adjust

// TODO: no consensus start/end/middle of seasons
// TODO: dateext
// march 24.5
// june 25
// sep 25
// dec 24.5

impl Angles {
    // Assumes +Y is north.
    // seems good, checked in hammer and irl (scary)
    pub(crate) fn from_solar_pos(pos: SolarPos) -> Self {
        let pitch = pos.zenith_angle - 90.0;
        // angle right from north (azimuth) to angle left from +X/east (yaw)
        // rem_euclid() means slam into 0..360 range
        let yaw = (270.0 - pos.azimuth).rem_euclid(360.0);
        let roll = 0.0;
        Angles { pitch, yaw, roll }
    }
}

pub trait DateTimeUtcExt {
    /// Calculates the timezone offset to a second from a longitude east, and
    /// uses that as the offset for a [`DateTime<Utc>`].
    fn from_longitude(longitude_east: f64, datetime: NaiveDateTime) -> Option<DateTime<Utc>> {
        let offset = lon_to_offset(longitude_east)?;
        let datetime_fixed = DateTime::<FixedOffset>::from_local(datetime, offset);
        Some(datetime_fixed.into())
    }
}

impl DateTimeUtcExt for DateTime<Utc> {}

pub trait NaiveDateTimeExt {
    fn from_season(season: Season, time: NaiveTime) -> Option<NaiveDateTime> {
        let year = 2023;
        let (month, day) = match season {
            Season::Spring => (3, 24),  // March 24.5
            Season::Summer => (6, 25),  // June 25
            Season::Fall => (9, 25),    // Sep 25
            Season::Winter => (12, 24), // Dec 24/5
        };
        Some(NaiveDate::from_ymd_opt(year, month, day)?.and_time(time))
    }
}

impl NaiveDateTimeExt for NaiveDateTime {}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

/// Outputs closest `FixedOffset` "timezone" to the second.
/// Longitudes outside of the bounds -180 to 180 give erroneous results.
pub(crate) fn lon_to_offset(longitude_east: f64) -> Option<FixedOffset> {
    let tz_hour = longitude_east / 15.0;
    let tz_secs = (tz_hour * 3600.0).round() as i32;
    // casting f64 to i32 truncates and does clamping and stuff
    FixedOffset::east_opt(tz_secs)
}

// TODO: SolarPos to
/// Calculate the sun position from world position and a local date and time.
pub fn calc_solar_position_local(
    lat: f64,
    lon: f64,
    datetime: NaiveDateTime,
) -> Result<SolarPos, SpaError> {
    let utc = DateTime::from_longitude(lon, datetime).ok_or(SpaError::BadParam)?;
    spa::calc_solar_position(utc, lat, lon)
}
// TODO:LOC: TODO: make a lot better
/// Get the mao lighting for a location and time
pub fn loc_time_to_sun(
    lat: f64,
    lon: f64,
    datetime: NaiveDateTime,
) -> Result<GlobalLighting, SpaError> {
    let solar_pos = calc_solar_position_local(lat, lon, datetime)?;
    let mut sun_dir = Angles::from_solar_pos(solar_pos);
    dbg!(sun_dir.pitch);
    let RGB { r, g, b } = pitch_to_rgb(-sun_dir.pitch);
    sun_dir.pitch = -sun_dir.pitch.abs();

    Ok(GlobalLighting {
        sun_color: ColorBrightness::new(r, g, b, 255), // TODO: brightness
        sun_dir: sun_dir.clone(),
        amb_color: ColorBrightness::new(171, 206, 220, 50), // default l4d2
        amb_dir: sun_dir,
        dir_lights: Vec::new(),
    })
}

// Property::new("origin", "0 0 0"),
// Property::new("SunSpreadAngle", "0"),
// Property::new("pitch", "-14"),
// Property::new("angles", "0 30 0"),
// Property::new("_lightscaleHDR", "1"),
// Property::new("_lightHDR", "-1 -1 -1 1"),
// Property::new("_light", "228 215 192 400"),
// Property::new("_AmbientScaleHDR", "1"),
// Property::new("_ambientHDR", "-1 -1 -1 1"),
// Property::new("_ambient", "171 206 220 50"),
// Property::new("classname", "light_environment"),

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn loc_time() {
        let datetime =
            NaiveDate::from_ymd_opt(2023, 4, 21).unwrap().and_hms_opt(14, 42, 0).unwrap();
        // let datetime = chrono::Local::now().naive_local();
        println!("datetime: {}", datetime);
        // Las Vegas
        let lighting = loc_time_to_sun(36.188110, -115.176468, datetime).unwrap();
        let dir = &lighting.sun_dir;
        dbg!(dir);
        // verified in hammer/l4d2 and irl (scary)
        assert_relative_eq!(-46.0, dir.pitch, epsilon = 3.0);
        assert_relative_eq!(22.0, dir.yaw, epsilon = 3.0);
        assert_relative_eq!(0.0, dir.roll);
    }
}
