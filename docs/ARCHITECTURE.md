# Architecture — lib vs interaction-engine

- **`worldline` crate (`src/lib.rs:1`):** `std` + `serde` only (default). Exports `Epoch`, `constants::AU/GM_*`, `FrameId/Rotation3`, `SpacetimeMetric`, `Worldline`, `Integrator`, `HelioState`. No `bsp` data, no `6DOF`, no `HIL`. Hardened via `error::WorldlineError` + `try_*`.
- **Interaction-engine (your repo):** owns `PropulsionModel: AccelerationModel`, `Spacecraft {mass, area, Cr, Cd, quaternion}`, `Scheduler`, `HIL` `RT` loop. Consumes `worldline` as `cargo package` artifact (`dist/*.crate`).
- **Civilization engine (optional layer):** `ABM` + `economy` on top, calls `worldline::Ephemeris` for `proper_time_step` only — `MC` `UQ`, not deterministic polity.

```
interaction_engine ──uses──► worldline (epoch, frames, metric, integrator, astro)
       │                              ▲
       └──owns──► dynamics/drag/srp/thrust, 6DOF, atmosphere, control
```

Keep `worldline` `no_std`-ready (future) by feature-gating `spice/sofa`.
