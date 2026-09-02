//! # worldline
//!
//! Time-aware, fidelity-focused **relativistic** simulation library.
//!
//! `worldline` models trajectories as **worldlines** `τ → Event` parametrized
//! by **proper time** `τ`, with clean separation of proper vs coordinate
//! time via [`time::SimulationClock`] and Lorentz-aware transforms.
//!
//! ## Scope
//! - **Special relativity** — four-vectors, Minkowski metric, Lorentz boosts,
//!   time dilation / length contraction, inertial & sampled worldlines.
//! - **GR scaffolding** — [`metric::SpacetimeMetric`] trait + [`metric::SchwarzschildMetric`]
//!   and [`metric::GeodesicMetric`] / [`worldline::GeodesicWorldline`] stubs.
//! - **J2000 / Solar-System validation** — [`time::Epoch`] (JD, TAI/TT/UTC/TDB/TCB),
//!   [`frame::j2000`] (EME2000/ICRS/ECLIPTIC), [`astro`] (DE440-sampled helio states
//!   at 2000-01-01 12:00 TT, `Body`, tolerances) for a "space lab" integration.
//! - **Hardened** — finite-input checks, `try_*` fallible constructors, `WorldlineError`,
//!   relative epsilons, `Result`-based APIs for superluminal/mass/infinite guards.
//! - **Library-only** — no server/networking in core; see `examples/server.rs`
//!   (`--features server`).
//!
//! ## Quick start
//! ```rust
//! use worldline::{
//!     astro::{j2000_heliocentric, Body},
//!     frame::{InertialFrame, Vec3},
//!     metric::{MinkowskiMetric, SpacetimeMetric},
//!     spacetime::Event,
//!     time::{Epoch, FidelityLevel, SimulationConfig},
//!     worldline::{InertialWorldline, Worldline},
//! };
//!
//! let origin = Event::at_origin();
//! let v = Vec3::new(1e7, 0.0, 0.0);
//! let wl = InertialWorldline::try_from_velocity(origin, v).unwrap();
//! let e = wl.event_at_proper_time(1.0);
//! assert!(e.ct() > 0.0);
//!
//! let mink = MinkowskiMetric::mostly_minus();
//! let d = origin.displacement_to(&e);
//! let _is_timelike = mink.proper_time_interval(&d);
//!
//! let earth = j2000_heliocentric(Body::Earth);
//! let epoch = Epoch::j2000();
//! assert!((epoch.jd_tt - 2451545.0).abs() < 1e-9);
//! ```
//!
//! Feature flags: `server` (tokio + clap + tracing, for the example),
//! `async` (async `SimulationEngine`).

pub mod astro;
pub mod constants;
pub mod error;
pub mod frame;
pub mod integrator;
pub mod metric;
pub mod particle;
pub mod simulation;
pub mod spacetime;
pub mod time;
pub mod worldline;

// Re-exports for ergonomic root access
pub use astro::{Body, HelioState, j2000_heliocentric};
pub use constants::{AU, C, C2, G, GM_EARTH, GM_SUN, INV_C, INV_C2, JD_J2000};
pub use error::{Result, WorldlineError};
pub use frame::{InertialFrame, Observer, Rotation3, Vec3};
pub use metric::{
    CausalCharacter, GeodesicMetric, MinkowskiMetric, SchwarzschildMetric, Signature,
    SpacetimeMetric,
};
pub use particle::Particle;
#[cfg(feature = "async")]
pub use simulation::SimulationEngine;
pub use simulation::{Engine, EngineState, SimEngine};
pub use spacetime::{Event, FourVector};
pub use time::{
    ClockState, Epoch, FidelityLevel, SimulationClock, SimulationConfig, TimeScale, TimeStep,
};
pub use worldline::{GeodesicWorldline, InertialWorldline, SampledWorldline, Worldline};

// Backward compat shim — `worldline` was previously `simengine`
#[doc(hidden)]
pub mod simengine_compat {
    pub use crate::simulation::Engine as SimEngine;
}
