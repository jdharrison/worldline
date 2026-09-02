# worldline

Time-aware, fidelity-focused **relativistic simulation library** — SR with GR scaffolding, J2000-validated.

`worldline` models trajectories as **worldlines** `τ → Event` parametrized by **proper time** `τ`, with clean separation of proper vs coordinate time. Hardened for import into a custom interaction engine.

## Library core

- `spacetime` — `FourVector(ct,x,y,z)` + `Event`, finite-checked, `try_new` variants
- `metric` — `SpacetimeMetric` trait (relative lightlike epsilon), `MinkowskiMetric` (± signature), `SchwarzschildMetric` + `GeodesicMetric` scaffolding
- `frame` — `Vec3` (finite-checked), `InertialFrame` (`try_new`, `try_boost`, `try_gamma`), `Rotation3`, `FrameId` (ICRS/EME2000/GCRS/BCRS), `frame::j2000` (EME2000↔ecliptic, ICRS≈EME2000)
- `worldline` — `Worldline` trait, `InertialWorldline::try_from_velocity`, `SampledWorldline::try_new`, `GeodesicWorldline<M>`
- `particle` — `Particle::try_from_velocity`, `try_photon`, `is_on_shell` (relative tolerance)
- `time` — `FidelityLevel` (integration fidelity), `SimulationConfig::validate`/`try_with_fidelity`, `SimulationClock` (wall-paced / as-fast-as-possible, pause-aware), **`time::Epoch`** (J2000 = JD 2451545.0 TT, JD_TT/TAI/UTC/TDB/TCB, leap-second table to 2017, TDB periodic ≈1.6 ms)
- `integrator` — `Integrator` trait, `Leapfrog` (symplectic) + `RK4`, `CentralGravity` / `NBodyGravity`, `proper_time_step`
- `astro` — `Body`, `HelioState` (AU, AU/day), `j2000_heliocentric` (DE440-sampled heliocentric EME2000 at J2000 for 11 bodies), `within_tolerance`, `GM_SUN` etc.
- `constants` — `C`, `AU` (IAU 2012 exact), `GM_SUN`/`GM_EARTH`/`GM_MOON` (DE440), `JD_J2000`, `OBLIQUITY_J2000`, `SCHWARZSCHILD_RADIUS_*`
- `error` — `WorldlineError` / `Result`, `check_finite`, superluminal/mass/bounds guards
- `simulation` — sync `Engine::try_new` (validated), `SimEngine` alias; optional `async` feature (`SimulationEngine`)

Pure library: no networking. See `examples/server.rs` (UDP demo) and `examples/spacelab.rs` (J2000 solar-system lab).

## Quick start

```rust
use worldline::{
    astro::{j2000_heliocentric, Body},
    frame::Vec3,
    metric::{MinkowskiMetric, SpacetimeMetric},
    spacetime::Event,
    time::Epoch,
    worldline::{InertialWorldline, Worldline},
};

let origin = Event::at_origin();
let v = Vec3::new(1e7, 0.0, 0.0);
let wl = InertialWorldline::try_from_velocity(origin, v).unwrap();
let e = wl.event_at_proper_time(1.0);
let mink = MinkowskiMetric::mostly_minus();
assert!(mink.proper_time_interval(&origin.displacement_to(&e)).is_some());

let epoch = Epoch::j2000();
let earth = j2000_heliocentric(Body::Earth);
assert!((epoch.jd_tt - 2451545.0).abs() < 1e-12);
```

## Interaction-engine import (hardened)

```toml
[dependencies]
worldline = { path = "../worldline" } # or version = "0.1"
```

```rust
use worldline::{astro::{j2000_heliocentric, Body}, constants::AU, frame::Vec3,
                integrator::{CentralGravity, Leapfrog, NewtonianState, Integrator},
                time::Epoch, error::Result};

fn step_lab(state: &mut NewtonianState, t: f64, dt: f64) -> Result<()> {
    let grav = CentralGravity { gm: worldline::constants::GM_SUN };
    Leapfrog.step(state, t, dt, &grav)
}

let epoch = Epoch::j2000(); // 2000-01-01 12:00 TT
let s = j2000_heliocentric(Body::Earth);
let pos = Vec3::new(s.pos_au[0]*AU, s.pos_au[1]*AU, s.pos_au[2]*AU);
let vel = Vec3::new(s.vel_au_per_day[0]*AU/86400.0, s.vel_au_per_day[1]*AU/86400.0, s.vel_au_per_day[2]*AU/86400.0);
let mut lab_state = NewtonianState::new(pos, vel, worldline::constants::M_EARTH);
// t is seconds since J2000 in TDB/TT; proper_time_step can map to SR proper time.
```

Validation tests live in `tests/j2000_validation.rs` (JD, leap seconds, TDB <2 ms, helio distances, 30-day propagation). Run `cargo test` and `cargo run --example spacelab`.

## J2000 validation

- **Epoch**: `Epoch::from_tt_datetime(2000,1,1,12,0,0)` → JD_TT 2451545.0; `jd_tai`/`jd_utc` (TAI-UTC=32 at J2000), `jd_tdb` (periodic <2 ms), `jd_tcb` (L_B drift).
- **Helio states**: DE440 EME2000 at J2000 (AU, AU/day) for Sun→Pluto; `within_tolerance(pos 1e-6 AU≈150 km, vel 1e-8 AU/d≈1.7 mm/s)`.
- **Frames**: `frame::j2000::to_ecliptic` / `to_equatorial` (obliquity 23.439°), `ICRS≈EME2000` to 20 mas.
- **Integrators**: `Leapfrog` preserves 1 AU circular orbit to <2% over 365 days (dt=1 d); `RK4` <0.1% over 10 days (dt=1 h).

See `src/astro/mod.rs` and `src/time/epoch.rs` for sources (DE440, IAU 2015/2006).

## Features

- default: pure `std` + `serde`
- `server` — `tokio` + `clap` + `tracing` (examples/server)
- `async` — async `SimulationEngine`

## Examples

```bash
cargo run --example spacelab
cargo run --example server --features server -- --port 8080 --fidelity high
cargo test --test j2000_validation # J2000 solar-system suite
```

## Fidelity

| Level  | steps/s | max entities |
|--------|---------|--------------|
| Low    | 10      | 100          |
| Medium | 30      | 1 000        |
| High   | 60      | 10 000       |
| Ultra  | 120     | 50 000       |

## Hardening notes

- All public constructors have `try_*` fallible variants; legacy `new/from_*` retain INF/NaN propagation for compat but `try_*` should be used in lab code.
- `WorldlineError` covers superluminal, non-finite, empty worldline, duplicate tau, invalid mass/gamma, time-scale, metric/time errors.
- `SimulationConfig::validate` checks `steps>0`, `multiplier` finite `(0,1e9)`.
- `metric::causal_character` uses relative epsilon `1e-12*(|a||b|+|s2|)`.
- SR checks: `beta<1` enforced via `try_gamma_from_beta`; `MinkowskiMetric` etc. check `is_finite`.

## GR scaffolding

`SpacetimeMetric` + `GeodesicMetric::christoffel` + `GeodesicWorldline<M>` / `SchwarzschildMetric` are intentional stubs. Future geodesic integrators will be generic over `M: GeodesicMetric` without breaking API.

## License

MIT
