# Roadmap — timeline & artifact flow

```
M01 (DONE v0.1.0) ──► M02 ephemeris+time ──► M03 frames ──► M04 gravity ─┬─► M05 drag/SRP
                    (2-4PM)              (2-3PM)       (4-6PM)        │   (3-4PM)
                                                                   └─► M06 PPN (2-3PM)
                                                                            │
M07 DOP853 ──────────────────────────────────────────────────────────────────┘ (2-3PM)
  │
M08 V&V/nightly (continuous)
```

## Artifact flow (now)

`git push main` → `.github/workflows/ci.yml:1` `validate` (`fmt --check`, `clippy -D warnings`, `check`, `test --all-features`, `doc`, `package`, `build --release`) → `artifact` (`dist/*.crate`, `dist/doc`, `dist/*.rlib`, `dist/server|spacelab`) via `actions/upload-artifact@v4` 30d.

Local dry-run:
```bash
cargo fmt --check && cargo clippy --all-features --examples -- -D warnings
cargo test --all-features && cargo doc --no-deps --all-features
cargo package && cargo build --release --all-features --examples
ls -lh target/package/*.crate target/doc/worldline/index.html target/release/libworldline.rlib
```

## Versioning

- `0.1.x` `M01` scaffolding
- `0.2.0` `M02+M03` (`Ephemeris` API break)
- `0.3.0` `M04+M05` (force models)
- `0.4.0` `M06+M07` (PPN+DOP853)
- `1.0.0` `M08` operational `TRL 6` 1yr `<10m`

## Dependencies (no vendor lock)

`worldline` stays `std` + `serde` (default). `spice/anise`, `hifitime`, `SOFA`/`erfa`, `nalgebra` feature-gated (`spice`, `sofa`).

## Risk

Largest risk: `DE440` vendor + `IERS` updates breaking `TDB` µs → gate `M02` first, keep `M01` pure as fallback `j2000_heliocentric` at `1e-6 AU`.
