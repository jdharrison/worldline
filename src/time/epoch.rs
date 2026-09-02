//! J2000 epoch, Julian Dates, and astronomical time scales.
//!
//! Implements a hardened subset of time metrology needed for solar-system
//! validation:
//!
//! - **J2000** = 2000-01-01 12:00:00 TT = JD 2451545.0
//! - Scales: TAI, TT, UTC (leap seconds), TDB (approx), TCB (linear placeholder)
//! - Julian Date conversions (TT-based, as used by ephemerides)
//! - Leap second table (TAI-UTC) up to 2017-01-01 — extend as needed
//!
//! TT is the ephemeris argument; for many apps TDB ≈ TT to <2ms (periodic
//! term). This library provides the IAU 2006 `TDB = TT + periodic` scaffolding
//! and a linear TCB stub. Consumers needing microsecond TDB should supply
//! their own `tcb_from_tt` if required.

use crate::constants::{JD_J2000, SECONDS_PER_DAY, TT_MINUS_TAI};
use crate::error::{Result, WorldlineError};

/// Leap second table: (JD_UTC at new offset, TAI-UTC seconds).
/// Sorted ascending. Last entry is 2017-01-01: offset 37s.
/// Extend for future leap seconds.
const LEAP_TABLE: &[(f64, f64)] = &[
    (2_441_317.5, 10.0), // 1972-01-01
    (2_441_499.5, 11.0), // 1972-07-01
    (2_441_683.5, 12.0), // 1973-01-01
    (2_442_048.5, 13.0), // 1974-01-01
    (2_442_413.5, 14.0), // 1975-01-01
    (2_442_778.5, 15.0), // 1976-01-01
    (2_443_144.5, 16.0), // 1977-01-01
    (2_443_509.5, 17.0), // 1978-01-01
    (2_443_874.5, 18.0), // 1979-01-01
    (2_444_239.5, 19.0), // 1980-01-01
    (2_444_786.5, 20.0), // 1981-07-01
    (2_445_151.5, 21.0), // 1982-07-01
    (2_445_516.5, 22.0), // 1983-07-01
    (2_446_247.5, 23.0), // 1985-07-01
    (2_447_161.5, 24.0), // 1988-01-01
    (2_447_892.5, 25.0), // 1990-01-01
    (2_448_257.5, 26.0), // 1991-01-01
    (2_448_804.5, 27.0), // 1992-07-01
    (2_449_169.5, 28.0), // 1993-07-01
    (2_449_534.5, 29.0), // 1994-07-01
    (2_450_083.5, 30.0), // 1996-01-01
    (2_450_630.5, 31.0), // 1997-07-01
    (2_451_179.5, 32.0), // 1999-01-01
    (2_453_736.5, 33.0), // 2006-01-01
    (2_454_832.5, 34.0), // 2009-01-01
    (2_456_109.5, 35.0), // 2012-07-01
    (2_457_204.5, 36.0), // 2015-07-01
    (2_457_754.5, 37.0), // 2017-01-01
];

fn tai_minus_utc_at_jd_utc(jd_utc: f64) -> f64 {
    let mut off = 0.0;
    for (jd, o) in LEAP_TABLE {
        if jd_utc >= *jd {
            off = *o;
        } else {
            break;
        }
    }
    if off == 0.0 {
        // Before table: assume 10s initial or error? Return NaN sentinel via caller.
        0.0
    } else {
        off
    }
}

/// Enumerated time scales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeScale {
    /// International Atomic Time.
    TAI,
    /// Terrestrial Time (TT = TAI + 32.184s).
    TT,
    /// Coordinated Universal Time (leap seconds).
    UTC,
    /// Barycentric Dynamical Time (TDB ≈ TT + periodic <2ms).
    TDB,
    /// Barycentric Coordinate Time (linear drift ~ 1.5505e-8).
    TCB,
}

/// An epoch in TT (with lazy conversions).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Epoch {
    /// Julian Date in TT.
    pub jd_tt: f64,
}

impl Epoch {
    /// J2000 epoch: 2000-01-01 12:00:00 TT.
    pub fn j2000() -> Self {
        Self { jd_tt: JD_J2000 }
    }

    pub fn from_jd_tt(jd_tt: f64) -> Result<Self> {
        if !jd_tt.is_finite() {
            return Err(WorldlineError::TimeError(format!(
                "non-finite JD_TT {}",
                jd_tt
            )));
        }
        // Reasonable bounds: JD 0 .. 10M
        if jd_tt < 0.0 || jd_tt > 10_000_000.0 {
            return Err(WorldlineError::OutOfBounds {
                what: "JD_TT".into(),
                value: jd_tt,
                min: 0.0,
                max: 10_000_000.0,
            });
        }
        Ok(Self { jd_tt })
    }

    pub fn from_jd_tai(jd_tai: f64) -> Result<Self> {
        // TT = TAI + 32.184s
        Self::from_jd_tt(jd_tai + TT_MINUS_TAI / SECONDS_PER_DAY)
    }

    pub fn from_jd_utc(jd_utc: f64) -> Result<Self> {
        if !jd_utc.is_finite() {
            return Err(WorldlineError::TimeError("non-finite JD_UTC".into()));
        }
        let off = tai_minus_utc_at_jd_utc(jd_utc);
        let jd_tai = jd_utc + off / SECONDS_PER_DAY;
        Self::from_jd_tai(jd_tai)
    }

    /// Construct from calendar date in TT (proleptic Gregorian).
    /// Month 1..12, day 1..31. Hour etc 0..60.
    pub fn from_tt_datetime(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: f64,
    ) -> Result<Self> {
        let jd = calendar_to_jd(year, month, day, hour, minute, second)?;
        Self::from_jd_tt(jd)
    }

    /// Seconds since J2000 in TT.
    pub fn seconds_since_j2000_tt(&self) -> f64 {
        (self.jd_tt - JD_J2000) * SECONDS_PER_DAY
    }

    /// Days since J2000 in TT.
    pub fn days_since_j2000_tt(&self) -> f64 {
        self.jd_tt - JD_J2000
    }

    /// Julian centuries since J2000 in TT.
    pub fn centuries_since_j2000_tt(&self) -> f64 {
        self.days_since_j2000_tt() / 36525.0
    }

    pub fn jd_tai(&self) -> f64 {
        self.jd_tt - TT_MINUS_TAI / SECONDS_PER_DAY
    }

    pub fn jd_utc(&self) -> f64 {
        // Iterate to find correct leap offset (depends on UTC JD itself).
        // Use TAI JD as initial guess for UTC JD.
        let jd_tai = self.jd_tai();
        // Initial guess: subtract current TAI-UTC at TT.
        // We do two iterations: get offset at guess, recompute.
        let mut jd_utc_guess = jd_tai - tai_minus_utc_at_jd_utc(jd_tai) / SECONDS_PER_DAY;
        for _ in 0..2 {
            let off = tai_minus_utc_at_jd_utc(jd_utc_guess);
            jd_utc_guess = jd_tai - off / SECONDS_PER_DAY;
        }
        jd_utc_guess
    }

    /// TDB approximation: TT + 0.001657 * sin(628.3076*T + 6.2401) + ... (<2ms).
    /// This is the leading term from IAU 2006. For full accuracy plug in DE ephemeris.
    pub fn jd_tdb(&self) -> f64 {
        let t = self.centuries_since_j2000_tt();
        // Leading periodic term amplitude ~1.657ms -> 1.9e-8 days
        let periodic_days = 0.001657 * (628.3076 * t + 6.2401).sin() / SECONDS_PER_DAY;
        self.jd_tt + periodic_days
    }

    /// TCB approximation: TCB = TDB + L_B * (JD_TDB - 2443144.5)*86400 etc.
    /// Linear drift L_B = 1.550519768e-8. Returns JD_TCB.
    pub fn jd_tcb(&self) -> f64 {
        const L_B: f64 = 1.550519768e-8;
        const T0_JD_TDB: f64 = 2_443_144.5; // 1977-01-01 00:00 TAI ~ TCB zero
        let tdb = self.jd_tdb();
        let dt = (tdb - T0_JD_TDB) * SECONDS_PER_DAY;
        tdb + L_B * dt / SECONDS_PER_DAY
    }

    /// Convert to another scale (returns JD in that scale).
    pub fn jd_in(&self, scale: TimeScale) -> f64 {
        match scale {
            TimeScale::TT => self.jd_tt,
            TimeScale::TAI => self.jd_tai(),
            TimeScale::UTC => self.jd_utc(),
            TimeScale::TDB => self.jd_tdb(),
            TimeScale::TCB => self.jd_tcb(),
        }
    }
}

// Meeus / Explanatory Supplement algorithm for JD at 0h TT not needed — we use noon.
fn calendar_to_jd(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64) -> Result<f64> {
    if !(1..=12).contains(&month) {
        return Err(WorldlineError::TimeError(format!("month {}", month)));
    }
    if !(1..=31).contains(&day) {
        return Err(WorldlineError::TimeError(format!("day {}", day)));
    }
    if hour > 23 {
        return Err(WorldlineError::TimeError(format!("hour {}", hour)));
    }
    if minute > 59 {
        return Err(WorldlineError::TimeError(format!("minute {}", minute)));
    }
    if !(0.0..61.0).contains(&second) {
        return Err(WorldlineError::TimeError(format!("second {}", second)));
    }
    if !second.is_finite() {
        return Err(WorldlineError::TimeError("second non-finite".into()));
    }
    let mut y = year;
    let mut m = month as i32;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    let jd0 = (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day as f64
        + b
        - 1524.5;
    let frac = (hour as f64 + minute as f64 / 60.0 + second / 3600.0) / 24.0;
    Ok(jd0 + frac)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{JD_J2000, SECONDS_PER_DAY};

    #[test]
    fn j2000_seconds_zero() {
        let e = Epoch::j2000();
        assert!((e.seconds_since_j2000_tt()).abs() < 1e-9);
        assert_eq!(e.jd_tt, JD_J2000);
    }

    #[test]
    fn jd_roundtrip_tt() {
        let e = Epoch::from_tt_datetime(2000, 1, 1, 12, 0, 0.0).unwrap();
        assert!((e.jd_tt - JD_J2000).abs() < 1e-9);
    }

    #[test]
    fn leap_seconds_j2000() {
        let e = Epoch::j2000();
        // At J2000, TAI-UTC = 32 (from table)
        let off = tai_minus_utc_at_jd_utc(e.jd_utc());
        assert!((off - 32.0).abs() < 1e-9);
    }

    #[test]
    fn tdb_within_2ms() {
        let e = Epoch::j2000();
        let diff = (e.jd_tdb() - e.jd_tt) * SECONDS_PER_DAY;
        assert!(diff.abs() < 0.002);
    }

    #[test]
    fn calendar_invalid() {
        assert!(Epoch::from_tt_datetime(2000, 13, 1, 0, 0, 0.0).is_err());
    }
}
