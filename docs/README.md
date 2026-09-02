# docs — worldline roadmap and hardening

This folder is the source of truth for pushing `worldline` from J2000 lab scaffolding (now, `v0.1.0` `src/lib.rs:1`) toward NASA/aero operational fidelity and toward the space-travel / civilization simulation foundation.

- **GAPS.md** — what is stub vs required for `DE440`/`SOFA`-level fidelity. No hand-waving.
- **MILESTONES.md** — phased milestones `M01…M08` with scope, tasks, acceptance, effort.
- **ROADMAP.md** — timeline, dependencies, artifact strategy (crate `worldline-*.crate` + `target/doc` via `.github/workflows/ci.yml:1`).
- **ARCHITECTURE.md** — how `worldline` stays `std` lib and interaction-engine owns vehicle/6DOF/HIL.
- **`milestones/`** — per-milestone one-pagers (copy of `MILESTONES.md` slices for issue tracking).

Validated state now: `cargo test --all-features` `35+8` pass, `cargo clippy --all-features --examples -- -D warnings` clean, `cargo package` `worldline-0.1.0.crate` `src/time/epoch.rs:88` `TT-TAI=32.184`, `src/astro/mod.rs:1` `DE440` epoch sample, `src/integrator/mod.rs:1` `Leapfrog/RK4`.

Start here: `MILESTONES.md` `M01` is done — `M02` is the next gate (`ephemeris+time+frames` to `<10m` 1yr).
