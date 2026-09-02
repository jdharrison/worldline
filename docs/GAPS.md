# Gaps — worldline `v0.1.0` vs operational fidelity

> Hard truth: `v0.1.0` is TRL 2-3 lab scaffolding. Operational (GMAT/MONTE/SPICE, STK, JSBSim `DO-178C`) is TRL 8-9.

| Subsystem | Now (`src/*`) | Required | Delta |
|---|---|---|---|
| **Ephemeris** `src/astro/mod.rs:1` | 11× `HelioState` at `JD_TT 2451545.0`, `1e-6 AU (150km)` | Full `DE440/441` Chebyshev `.bsp` any `TDB` to `<1m`/`<1mm/s`, asteroids | `+vendor spice/anise, trait Ephemeris` |
| **Time** `src/time/epoch.rs:1` | `TAI/TT/UTC/TDB/TCB` `TDB≈TT+1.657ms sin` (<2ms), leap `1972-2017` | `µs` `TDB-TT` (127 terms IAU2006), `TCB` secular `L_B`, `UT1-UTC/DUT1`, `IERS C04` | `+hifitime/SOFA` |
| **Frames** `src/frame/mod.rs:1` | `ICRS≈EME2000` identity, `EME2000↔ecliptic ε` | `ICRS→GCRF (bias) →ITRS (IAU2006/2000A pre nut) →TEME/ITRF+EOP`, `GMST`, `quat` | `+SOFA/erfa` |
| **Gravity** `src/integrator/mod.rs:22` | `CentralGravity/NBody` point-mass `a=-GM r/r³` | `J2/Jn` `EGM2008 20×20` + tides | `+SphericalHarmonics` |
| **Non-grav** | none | `SRP+eclipse`, `NRLMSISE-00` drag, thrust/mass-flow | `+drag/srp` crate |
| **Relativity** `src/metric/mod.rs:1` | `Minkowski + Schwarzschild` stub (`dot→Minkowski`, `christoffel=0`) | PPN EIH + Lense-Thirring | `+Γ(u) integrator` |
| **Integrator** `src/integrator/mod.rs:10` | Fixed `Leapfrog/RK4` `dt=3600s` `r<2%` 30d | Variable `DOP853`/`ABM` `atol 1e-12` `1yr <10m` | `+dop853` |
| **6DOF/Aero** | 3DOF point-mass | `6DOF` quaternion `6×6` + `US76` aero `C_L/D` | Interaction-engine owns |
| **V&V** `tests/j2000_validation.rs:1` | 35 unit + 8 coarse (30-day `r`) | `1yr` `DE440` `±10m` regression + covariance/MC | `+de440_1yr` test |
| **SWE/RT** `.github/workflows/ci.yml:1` | `fmt/clippy/test/doc/package` + `crate/docs/rlib` artifacts | `DO-178C` trace, `HIL`, `RT`, `EOP` nightly | `+cache de440` |

## Why it matters for your two use-cases

- **Space travel (fusion→c):** `γ` already `try_gamma_from_velocity` guarded (`src/frame/mod.rs:64`); `1AU` circular `1yr` `<2%` fails `station-keeping` → needs gap 4+6.
- **Civilization:** physics `±km` kept; social `±decades`. Don't claim deterministic polity — `MC+UQ` over `worldline::Epoch` only.

See `MILESTONES.md` for the closure tasks per gap.
