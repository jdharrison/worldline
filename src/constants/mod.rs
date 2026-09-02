#![allow(clippy::excessive_precision)]
//! Physical and astronomical constants (SI, IAU 2015 / DE440 where noted).

// Speed of light in vacuum, m/s (exact, SI).
pub const C: f64 = 299_792_458.0;
pub const C2: f64 = C * C;
pub const INV_C: f64 = 1.0 / C;
pub const INV_C2: f64 = 1.0 / C2;

/// Gravitational constant, m^3 kg^-1 s^-2 (CODATA 2018).
pub const G: f64 = 6.67430e-11;

/// Electron rest mass, kg.
pub const M_ELECTRON: f64 = 9.10938356e-31;

/// Proton rest mass, kg.
pub const M_PROTON: f64 = 1.67262192369e-27;

// ---------------------------------------------------------------------------
// Astronomical constants — IAU 2015 nominal / DE440 values.
// Source: IAU 2015 Resolution B3, DE440 ephemeris, and
// https://ssd.jpl.nasa.gov/astro_par.html
// ---------------------------------------------------------------------------

/// Astronomical Unit in meters (IAU 2012 exact).
pub const AU: f64 = 149_597_870_700.0;
pub const INV_AU: f64 = 1.0 / AU;

/// Julian Day for J2000 epoch: 2000-01-01 12:00 TT (JD 2451545.0).
pub const JD_J2000: f64 = 2_451_545.0;

/// Seconds per Julian day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// Days per Julian century (36525).
pub const DAYS_PER_CENTURY: f64 = 36_525.0;

/// Seconds per Julian century.
pub const SECONDS_PER_CENTURY: f64 = DAYS_PER_CENTURY * SECONDS_PER_DAY;

/// TT - TAI offset at J2000 (and by definition constant): 32.184 s.
pub const TT_MINUS_TAI: f64 = 32.184;

/// Gravitational parameter GM for Sun, m^3 s^-2 (DE440: 1.32712440041279419e20).
pub const GM_SUN: f64 = 1.32712440041279419e20;

/// GM for Earth (geocentric), m^3 s^-2 (DE440: 3.9860043543609598e14).
pub const GM_EARTH: f64 = 3.9860043543609598e14;

/// GM for Moon, m^3 s^-2 (DE440: 4.902800118460361e12).
pub const GM_MOON: f64 = 4.902800118460361e12;

/// Mass of Earth, kg (derived).
pub const M_EARTH: f64 = GM_EARTH / G;

/// Mass of Moon, kg.
pub const M_MOON: f64 = GM_MOON / G;

/// Mass of Sun, kg (derived).
pub const M_SUN: f64 = GM_SUN / G;

/// Mean obliquity of the ecliptic at J2000, degrees (IAU 2006).
pub const OBLIQUITY_J2000_DEG: f64 = 23.439279444444445;
pub const OBLIQUITY_J2000_RAD: f64 = OBLIQUITY_J2000_DEG * std::f64::consts::PI / 180.0;

/// Light-time for 1 AU, seconds (AU / c). ~499.004782 s.
pub const LIGHT_TIME_AU: f64 = AU * INV_C;

/// Schwarzschild radius of Sun, m (2GM/c^2).
pub const SCHWARZSCHILD_RADIUS_SUN: f64 = 2.0 * GM_SUN * INV_C2;

/// Schwarzschild radius of Earth, m.
pub const SCHWARZSCHILD_RADIUS_EARTH: f64 = 2.0 * GM_EARTH * INV_C2;
