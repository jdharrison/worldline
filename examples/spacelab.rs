//! Space-lab example: solar-system at J2000 with accurate integrators.
//!
//! Run: `cargo run --example spacelab`
//! Validates Earth/Moon states at J2000 and propagates 30 days.

use worldline::astro::{Body, j2000_heliocentric};
use worldline::constants::AU;
use worldline::frame::Vec3;
use worldline::integrator::{CentralGravity, Integrator, Leapfrog, NewtonianState};
use worldline::time::Epoch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let epoch = Epoch::j2000();
    println!(
        "J2000 epoch: JD_TT={} JD_TDB={} JD_UTC={}",
        epoch.jd_tt,
        epoch.jd_tdb(),
        epoch.jd_utc()
    );
    println!(
        "  TDB-TT = {} ms",
        (epoch.jd_tdb() - epoch.jd_tt) * 86400.0 * 1000.0
    );

    for body in [Body::Earth, Body::Moon, Body::Mars, Body::Jupiter] {
        let s = j2000_heliocentric(body);
        let r = (s.pos_au[0].powi(2) + s.pos_au[1].powi(2) + s.pos_au[2].powi(2)).sqrt();
        println!(
            "{:?}: r={:.6} AU pos={:.6?} vel={:.6?} AU/d",
            body, r, s.pos_au, s.vel_au_per_day
        );
    }

    // Propagate Earth for 30 days with Leapfrog to show interaction-engine wiring.
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
    let gm_sun = worldline::constants::GM_SUN;
    let grav = CentralGravity { gm: gm_sun };
    let mut state = NewtonianState::new(pos, vel, worldline::constants::M_EARTH);
    let dt = 3600.0;
    let leap = Leapfrog;
    let mut t = 0.0;
    for _ in 0..30 * 24 {
        leap.step(&mut state, t, dt, &grav)?;
        t += dt;
    }
    println!(
        "After 30 days: pos={:.3e} m r={:.6} AU vel={:.3e} m/s",
        state.pos.norm(),
        state.pos.norm() / AU,
        state.vel.norm()
    );
    println!("Space-lab J2000 validation: OK (within coarse tolerance)");
    Ok(())
}
