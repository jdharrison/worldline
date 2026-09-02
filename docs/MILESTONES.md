# Milestones — worldline to operational fidelity

> Each milestone is shippable, artifact-wired (`.github/workflows/ci.yml:1` `dist/*.crate`, `dist/doc`, `dist/*.rlib`). `TRL` per NASA.

## M01 — J2000 Foundation (DONE, `v0.1.0`, `dd356af`)
- **Scope:** Pure `std` lib `worldline` (`src/lib.rs:1`), `spacetime`, `metric`, `frame`, `worldline`, `particle`, `time::Epoch`, `constants`, `error`, `integrator`, `astro`, `simulation`.
- **Artifacts:** `crate 37K` `target/package/worldline-0.1.0.crate`, `r_lib 702K`, `examples/spacelab|server`.
- **Validation:** `cargo test --all-features` `35+8` pass, `clippy -D warnings` clean (`examples/server.rs:170` `..Default::default()`), 30-day `r` `<2%`.
- **Docs:** `README.md` J2000 section, `src/time/epoch.rs:88` leap `32s` at `J2000`.

## M02 — Ephemeris + Time (next gate, 2-4 PM)
- **Now:** `src/astro/mod.rs:1` single-epoch; `src/time/epoch.rs:22` `TDB≈TT+1.657ms`.
- **Target:** `DE440` `.bsp` `Ephemeris` trait any `TDB` `<1m`; `TDB-TT` to `±1µs` + leap `>2017`.
- **Tasks:**
  1. `src/astro/ephemeris.rs` `trait Ephemeris: pos(body, epoch: Epoch)->Result<Vec3>`; `DE440Ephemeris` via `anise`/`spice`.
  2. Vendor `hifitime`/`SOFA` `iauTdbtt`; replace `LEAP_TABLE` with `iers` auto.
  3. `cargo test --test de440` `Horizons` `<1km` 2000-2030.
- **Accept:** `j2000_heliocentric` → `ephemeris.at(JD_TDB 2451545.0, Earth) <1km/1mm/s`.
- **Dep:** time `µs` before ephemeris.

## M03 — Frames + EOP (2-3 PM, depends M02)
- **Now:** `src/frame/mod.rs:174` `Rotation3::mul`, `j2000::to_ecliptic`.
- **Tasks:** Vendor `SOFA`/`erfa`; `src/frame/sofa.rs` `bias+precession+nutation(jd_tdb)->Mat3`; `FrameId::GCRF/ITRF/TEME`; `transform(v, from, to, epoch)`.
- **Accept:** `EME2000->GCRF` 20mas, `GCRF->ITRF` `<5mas` vs `SOFA`.

## M04 — Spherical Harmonics + Tides (4-6 PM, depends M02/M03)
- **Tasks:** `src/dynamics/gravity.rs` `SphericalHarmonics {c_nm,s_nm} ∇U`; `20×20` `EGM2008`; `GMAT` `J2` `<10m/day`.
- **Accept:** LEO `J2` drift `±2km/day` matches `GMAT` 24h `<10m`.

## M05 — Drag + SRP + Thrust (3-4 PM, parallel M04)
- **Tasks:** `src/dynamics/drag.rs` `NRLMSISE-00`, `src/dynamics/srp.rs` `Cr` + eclipse; `AccelerationModel::acceleration(state, epoch, params)->Result`.
- **Accept:** 400km LEO `1-day` `r` `±100m` vs `NRLMSISE`.

## M06 — Relativity (PPN) (2-3 PM, depends M04)
- **Tasks:** `GeodesicWorldline<M: GeodesicMetric>` `du/dτ=-Γ u u`, `SchwarzschildMetric::dot(pos,)` position-dependent `g_tt=-(1-rs/r)`, PPN `a = (GM/c²r³)[(4GM/r - v²)r+4(r·v)v]`.
- **Accept:** Mercury `43″/cy ±0.5″`, Earth `PPN` `3cm/yr`.

## M07 — Variable-Step Propagator (2-3 PM, depends M02-M06)
- **Tasks:** `DOP853`/`ABM` `atol 1e-12` `trait Integrator::step_adaptive`; `Engine::propagate(epoch, dt_max)->Worldline`.
- **Accept:** `1yr` Earth circular `<10m` central-only vs `DE440`.

## M08 — V&V, Docs, Ops (ongoing 0.5 PM/sprint, depends all)
- **Tasks:** `tests/de440_1yr.rs` `<10m` nightly, `cargo bench` `ΔE/E<1e-9`, CI cache `de440.bsp`, req trace `TRL 5-6`.
- **Artifacts:** `dist/*.crate` + `dist/doc` + `dist/*.rlib` retained 30d `.github/workflows/ci.yml:66`.
- **Accept:** Nightly `main` green on `ubuntu-latest` with real `bsp`.

## Backlog — Interaction-Engine Owned (outside `worldline`)
- `6DOF` quaternion, `US76` aero, control, `HIL`, `DO-178C`. `worldline` supplies `Vec3`/`FrameId`/`Epoch` only.

## How to use

- Track issues as `M02-01: DE440Ephemeris trait`, `M02-02: TDB µs`, etc. — file per milestone in `docs/milestones/`.
- Gate: no `M04` until `M02` `<1km`.

