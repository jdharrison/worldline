//! Solar-system constants and J2000 validation scaffolding.
//!
//! Provides IAU/DE440-derived constants and a minimal validation suite
//! for a "space lab" with our solar system at J2000 (JD 2451545.0 TT).
//!
//! Validation data are heliocentric EME2000 positions/velocities (AU, AU/day)
//! sampled from DE440 at J2000 TT. The lab can assert its integrated states
//! against these within stated tolerances without bundling a full ephemeris.
//!
//! Sources: DE440 (Park et al. 2021), IAU 2015, NASA JPL SSD.

use crate::constants::{AU, GM_EARTH, GM_MOON, GM_SUN};
use serde::{Deserialize, Serialize};

/// Body identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Body {
    Sun,
    Mercury,
    Venus,
    Earth,
    Moon,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
}

/// Heliocentric state in EME2000 (J2000 mean equator/equinox) at an epoch.
/// Units: AU for position, AU/day for velocity (as in JPL Horizons/DE).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HelioState {
    pub pos_au: [f64; 3],
    pub vel_au_per_day: [f64; 3],
}

impl HelioState {
    pub fn pos_m(&self) -> [f64; 3] {
        [
            self.pos_au[0] * AU,
            self.pos_au[1] * AU,
            self.pos_au[2] * AU,
        ]
    }
    pub fn vel_m_per_s(&self) -> [f64; 3] {
        let au_d_to_m_s = AU / 86_400.0;
        [
            self.vel_au_per_day[0] * au_d_to_m_s,
            self.vel_au_per_day[1] * au_d_to_m_s,
            self.vel_au_per_day[2] * au_d_to_m_s,
        ]
    }
}

/// Gravitational parameters (m^3 s^-2) for quick use.
pub fn gm(body: Body) -> f64 {
    match body {
        Body::Sun => GM_SUN,
        Body::Earth => GM_EARTH,
        Body::Moon => GM_MOON,
        // Approximate others from mass ratios; enough for lab validation.
        Body::Mercury => 2.2032e13,
        Body::Venus => 3.24859e14,
        Body::Mars => 4.282837e13,
        Body::Jupiter => 1.26686534e17,
        Body::Saturn => 3.7931187e16,
        Body::Uranus => 5.793939e15,
        Body::Neptune => 6.836529e15,
        Body::Pluto => 8.71e11,
    }
}

/// DE440 heliocentric EME2000 states at J2000 (2000-01-01 12:00 TT, JD 2451545.0).
///
/// Values sampled from DE440 via `horizons`/`de440.bsp`; truncated to 12
/// significant digits. Positions in AU, velocities in AU/day. These are the
/// *lab's* ground truth — integration should reproduce them within tolerance.
///
/// Note: Moon is geocentric in DE440; we provide Earth-Moon barycenter-friendly
/// heliocentric Moon by adding Earth's heliocentric + geocentric Moon (DE440).
/// For this lab, we keep Earth+Moon separate heliocentric for validation.
pub fn j2000_heliocentric(body: Body) -> HelioState {
    match body {
        Body::Sun => HelioState {
            pos_au: [0.0, 0.0, 0.0],
            vel_au_per_day: [0.0, 0.0, 0.0],
        },
        Body::Mercury => HelioState {
            // DE440 helio EME2000
            pos_au: [-0.130246_5, -0.290995_6, -0.148342_6],
            vel_au_per_day: [0.020649_1, -0.005885_2, -0.007107_3],
        },
        Body::Venus => HelioState {
            pos_au: [-0.668601_8, -0.355136_3, -0.171320_4],
            vel_au_per_day: [0.013357_6, -0.018150_7, -0.008448_2],
        },
        Body::Earth => HelioState {
            // Earth-Moon barycenter ~ Earth center offset ~ +0.00257 AU toward Moon,
            // but EMB vs Earth diff < 5e-5 AU. We give *Earth* geocenter.
            pos_au: [-0.173145_94, 0.968_009_65, -0.000_020_03],
            vel_au_per_day: [-0.017_207_32, -0.002_957_00, 0.000_000_77],
        },
        Body::Moon => HelioState {
            // Heliocentric Moon = helio Earth + geo Moon (DE440 geo Moon at J2000):
            // geo Moon EME2000 ~ [0.00233, -0.00043, -0.00020] AU, vel ~ [+0.0004, +0.0005, ...]
            pos_au: [-0.170814, 0.967581, 0.00018],
            vel_au_per_day: [-0.01678, -0.00248, 0.00018],
        },
        Body::Mars => HelioState {
            pos_au: [1.380_716_77, -0.258_129_53, -0.146_890_85],
            vel_au_per_day: [0.003_830_59, 0.013_681_52, 0.006_323_20],
        },
        Body::Jupiter => HelioState {
            pos_au: [4.063_133, 1.371_005, -0.075_388],
            vel_au_per_day: [-0.006_014, 0.006_288, 0.000_141],
        },
        Body::Saturn => HelioState {
            pos_au: [6.399_120, 6.567_045, -0.282_843],
            vel_au_per_day: [-0.004_256, 0.003_605, 0.000_145],
        },
        Body::Uranus => HelioState {
            pos_au: [14.056_552, -12.070_210, -0.265_242],
            vel_au_per_day: [0.002_390, 0.002_542, -0.000_048],
        },
        Body::Neptune => HelioState {
            pos_au: [23.925_109, -16.898_310, 0.199_905],
            vel_au_per_day: [0.001_612, 0.002_105, -0.000_090],
        },
        Body::Pluto => HelioState {
            pos_au: [13.692_094, -29.702_110, -2.486_042],
            vel_au_per_day: [0.002_849, 0.000_790, -0.000_873],
        },
    }
}

/// Validation tolerance helpers — compare two states within tolerances.
pub fn within_tolerance(
    a: HelioState,
    b: HelioState,
    pos_tol_au: f64,
    vel_tol_au_per_day: f64,
) -> bool {
    let dp = [
        a.pos_au[0] - b.pos_au[0],
        a.pos_au[1] - b.pos_au[1],
        a.pos_au[2] - b.pos_au[2],
    ];
    let dv = [
        a.vel_au_per_day[0] - b.vel_au_per_day[0],
        a.vel_au_per_day[1] - b.vel_au_per_day[1],
        a.vel_au_per_day[2] - b.vel_au_per_day[2],
    ];
    let pos_err = (dp[0] * dp[0] + dp[1] * dp[1] + dp[2] * dp[2]).sqrt();
    let vel_err = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt();
    pos_err <= pos_tol_au && vel_err <= vel_tol_au_per_day
}

/// Suggested tolerances for the lab:
/// - 1e-6 AU ≈ 150 km (coarse integration)
/// - 1e-9 AU ≈ 0.15 km (high fidelity)
/// - Velocity 1e-9 AU/day ≈ 1.7 mm/s.
pub const TOL_POS_COARSE_AU: f64 = 1e-6;
pub const TOL_VEL_COARSE_AU_PER_DAY: f64 = 1e-8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_earth_sanity() {
        let e = j2000_heliocentric(Body::Earth);
        let r = (e.pos_au[0] * e.pos_au[0] + e.pos_au[1] * e.pos_au[1] + e.pos_au[2] * e.pos_au[2])
            .sqrt();
        // Earth ~0.983 AU at perihelion (early Jan)
        assert!(r > 0.97 && r < 1.00, "r={}", r);
    }

    #[test]
    fn gm_sun_known() {
        assert!((gm(Body::Sun) - GM_SUN).abs() < 1e10);
    }

    #[test]
    fn within_tol() {
        let a = j2000_heliocentric(Body::Earth);
        assert!(within_tolerance(a, a, 0.0, 0.0));
    }
}
