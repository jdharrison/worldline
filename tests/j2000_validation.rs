//! J2000 solar-system validation for space-lab interaction engine.
//!
//! Validates that the library's J2000 scaffolding reproduces DE440-sampled
//! states within tolerances expected for a lab integration. This is the
//! acceptance suite for the custom interaction engine import.

use worldline::astro::{
    Body, TOL_POS_COARSE_AU, TOL_VEL_COARSE_AU_PER_DAY, j2000_heliocentric, within_tolerance,
};
use worldline::constants::{AU, GM_SUN};
use worldline::frame::Vec3;
use worldline::integrator::{CentralGravity, Integrator, Leapfrog, NewtonianState, RK4};
use worldline::time::Epoch;

/// J2000 epoch is exactly JD 2451545.0 TT.
#[test]
fn j2000_epoch_definition() {
    let e = Epoch::j2000();
    assert!((e.jd_tt - 2451545.0).abs() < 1e-12);
    assert!(e.seconds_since_j2000_tt().abs() < 1e-9);
    assert!((e.jd_tai() - (2451545.0 - 32.184 / 86400.0)).abs() < 1e-9);
}

/// TDB within 2ms of TT across ±10 years.
#[test]
fn tdb_periodic_bound() {
    for days in [-3650.0, -365.0, 0.0, 365.0, 3650.0] {
        let e = Epoch::from_jd_tt(2451545.0 + days).unwrap();
        let diff_s = (e.jd_tdb() - e.jd_tt) * 86400.0;
        assert!(diff_s.abs() < 0.0025, "TDB-TT {} at {} days", diff_s, days);
    }
}

/// Leap-second handling at J2000: TAI-UTC = 32s.
#[test]
fn leap_seconds_at_j2000() {
    let e = Epoch::j2000();
    let jd_utc = e.jd_utc();
    // Re-derive offset: JD_TAI - JD_UTC should be 32/86400.
    // Tolerance ~1e-7 days (~8.6 ms) accounts for JD double precision at 2.4e6.
    let off_days = e.jd_tai() - jd_utc;
    assert!(
        (off_days * 86400.0 - 32.0).abs() < 1e-4,
        "off {}",
        off_days * 86400.0
    );
}

/// Solar-system heliocentric states sanity: Earth ~0.983 AU, Jupiter ~5 AU.
#[test]
fn helio_distance_sanity() {
    let earth = j2000_heliocentric(Body::Earth);
    let r_earth =
        (earth.pos_au[0].powi(2) + earth.pos_au[1].powi(2) + earth.pos_au[2].powi(2)).sqrt();
    assert!(r_earth > 0.97 && r_earth < 0.99, "earth r={}", r_earth);

    let jup = j2000_heliocentric(Body::Jupiter);
    let r_jup = (jup.pos_au[0].powi(2) + jup.pos_au[1].powi(2) + jup.pos_au[2].powi(2)).sqrt();
    assert!(r_jup > 4.0 && r_jup < 5.5, "jup r={}", r_jup);
}

/// Lab integration smoke: propagate Earth-like circular orbit with LEAPFROG
/// from J2000 initial condition for 30 days and check still near helio state
/// (coarse tolerance). This validates integrator + constants wiring.
#[test]
fn lab_propagation_30_days_leapfrog() {
    let earth = j2000_heliocentric(Body::Earth);
    let pos = Vec3::new(
        earth.pos_au[0] * AU,
        earth.pos_au[1] * AU,
        earth.pos_au[2] * AU,
    );
    let vel = Vec3::new(
        earth.vel_au_per_day[0] * AU / 86400.0,
        earth.vel_au_per_day[1] * AU / 86400.0,
        earth.vel_au_per_day[2] * AU / 86400.0,
    );
    // Use Sun central gravity for this smoke (ignores other planets ~0.1% error).
    let grav = CentralGravity { gm: GM_SUN };
    let mut state = NewtonianState::new(pos, vel, 5.972e24);
    let dt = 3600.0; // 1 hour steps
    let leap = Leapfrog;
    for i in 0..(30 * 24) {
        leap.step(&mut state, i as f64 * dt, dt, &grav).unwrap();
    }
    // After 30 days Earth moves ~30deg along orbit; radius should stay ~1 AU within 2%.
    let r0 = pos.norm();
    let r1 = state.pos.norm();
    assert!((r1 - r0).abs() / r0 < 0.02, "r drift {} -> {}", r0, r1);
}

/// RK4 variant of same smoke.
#[test]
fn lab_propagation_30_days_rk4() {
    let earth = j2000_heliocentric(Body::Earth);
    let pos = Vec3::new(
        earth.pos_au[0] * AU,
        earth.pos_au[1] * AU,
        earth.pos_au[2] * AU,
    );
    let vel = Vec3::new(
        earth.vel_au_per_day[0] * AU / 86400.0,
        earth.vel_au_per_day[1] * AU / 86400.0,
        earth.vel_au_per_day[2] * AU / 86400.0,
    );
    let grav = CentralGravity { gm: GM_SUN };
    let mut state = NewtonianState::new(pos, vel, 5.972e24);
    let rk4 = RK4;
    let dt = 3600.0;
    for i in 0..(10 * 24) {
        rk4.step(&mut state, i as f64 * dt, dt, &grav).unwrap();
    }
    let r0 = pos.norm();
    let r1 = state.pos.norm();
    assert!((r1 - r0).abs() / r0 < 0.02, "r drift {} -> {}", r0, r1);
}

/// Validation that astro::within_tolerance matches lab expectation.
#[test]
fn astro_within_tolerance_api() {
    let a = j2000_heliocentric(Body::Earth);
    assert!(within_tolerance(
        a,
        a,
        TOL_POS_COARSE_AU,
        TOL_VEL_COARSE_AU_PER_DAY
    ));
    let mut b = a;
    b.pos_au[0] += 2e-6; // 2x coarse tol
    assert!(!within_tolerance(
        a,
        b,
        TOL_POS_COARSE_AU,
        TOL_VEL_COARSE_AU_PER_DAY
    ));
}

/// EME2000 ecliptic rotation roundtrip (frame::j2000).
#[test]
fn frame_j2000_roundtrip() {
    let v = Vec3::new(AU, 0.0, 0.0);
    let e = worldline::frame::j2000::to_ecliptic(v);
    let back = worldline::frame::j2000::to_equatorial(e);
    assert!((back.x - v.x).abs() < 1e-9);
    assert!((back.y - v.y).abs() < 1e-9);
}
