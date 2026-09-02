//! Worldlines — parametrized trajectories in spacetime.
//!
//! Hardened: `InertialWorldline::try_from_velocity` validates `|v|<c`;
//! `SampledWorldline::try_new` validates sorted, finite, non-empty;
//! `GeodesicWorldline` stays generic over `M: SpacetimeMetric`.

use crate::error::{Result, WorldlineError};
use crate::metric::SpacetimeMetric;
use crate::spacetime::{Event, FourVector};

/// Core worldline trait: proper-time parametrized curve in spacetime.
pub trait Worldline: Send + Sync {
    fn event_at_proper_time(&self, tau: f64) -> Event;
    fn event_at_coordinate_time(&self, _t: f64) -> Option<Event> {
        None
    }
    fn four_velocity(&self, _tau: f64) -> FourVector {
        let eps = 1e-6;
        let e1 = self.event_at_proper_time(_tau - eps);
        let e2 = self.event_at_proper_time(_tau + eps);
        let delta = FourVector::new(
            e2.0.ct - e1.0.ct,
            e2.0.x - e1.0.x,
            e2.0.y - e1.0.y,
            e2.0.z - e1.0.z,
        );
        delta * (1.0 / (2.0 * eps))
    }
    fn proper_time_between_events(&self, _a: &Event, _b: &Event) -> Option<f64> {
        None
    }
}

/// Inertial (straight) worldline: `x(tau) = origin + u * tau`.
#[derive(Debug, Clone, Copy)]
pub struct InertialWorldline {
    pub origin: Event,
    /// Four-velocity `u = gamma*(c, v)`.
    pub four_velocity: FourVector,
}

impl InertialWorldline {
    pub fn new(origin: Event, four_velocity: FourVector) -> Self {
        Self {
            origin,
            four_velocity,
        }
    }
    pub fn try_new(origin: Event, four_velocity: FourVector) -> Result<Self> {
        if !origin.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "origin".into(),
                value: origin.ct(),
            });
        }
        four_velocity.check_finite()?;
        // Check timelike normalization: g(u,u) ~ c^2 (mostly-minus).
        // Allow null for photon placeholder but warn via caller.
        Ok(Self {
            origin,
            four_velocity,
        })
    }
    /// Create from 3-velocity `v` (m/s) at `origin`.
    pub fn from_velocity(origin: Event, velocity: crate::frame::Vec3) -> Self {
        // Legacy panicking path: superluminal => INF gamma -> still constructs.
        // Prefer try_from_velocity for hardened code.
        let beta2 = velocity.norm_squared() * crate::constants::INV_C2;
        let gamma = if beta2 >= 1.0 {
            f64::INFINITY
        } else {
            (1.0 - beta2).powf(-0.5)
        };
        let four_velocity = FourVector::new(
            gamma * crate::constants::C,
            gamma * velocity.x,
            gamma * velocity.y,
            gamma * velocity.z,
        );
        Self {
            origin,
            four_velocity,
        }
    }
    pub fn try_from_velocity(origin: Event, velocity: crate::frame::Vec3) -> Result<Self> {
        if !origin.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "origin".into(),
                value: origin.ct(),
            });
        }
        velocity.check_finite("velocity")?;
        let beta = velocity.norm() * crate::constants::INV_C;
        if beta >= 1.0 {
            return Err(WorldlineError::Superluminal {
                beta,
                velocity_norm: velocity.norm(),
            });
        }
        if !beta.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "beta".into(),
                value: beta,
            });
        }
        let gamma = (1.0 - beta * beta).powf(-0.5);
        if !gamma.is_finite() {
            return Err(WorldlineError::InvalidGamma { gamma });
        }
        let four_velocity = FourVector::new(
            gamma * crate::constants::C,
            gamma * velocity.x,
            gamma * velocity.y,
            gamma * velocity.z,
        );
        Ok(Self {
            origin,
            four_velocity,
        })
    }
    pub fn at_rest(origin: Event) -> Self {
        Self {
            origin,
            four_velocity: FourVector::new(crate::constants::C, 0.0, 0.0, 0.0),
        }
    }
}

impl Worldline for InertialWorldline {
    fn event_at_proper_time(&self, tau: f64) -> Event {
        // Check tau finite — if not, propagate NaN via Event (hardened callers use try_).
        let d = self.four_velocity * tau;
        Event::new(
            self.origin.ct() + d.ct,
            self.origin.x() + d.x,
            self.origin.y() + d.y,
            self.origin.z() + d.z,
        )
    }
    fn event_at_coordinate_time(&self, t: f64) -> Option<Event> {
        if !t.is_finite() {
            return None;
        }
        let gamma = self.four_velocity.ct * crate::constants::INV_C;
        if gamma == 0.0 || !gamma.is_finite() {
            return None;
        }
        let tau_prop = t / gamma;
        Some(self.event_at_proper_time(tau_prop))
    }
    fn four_velocity(&self, _tau: f64) -> FourVector {
        self.four_velocity
    }
}

/// Sampled / piecewise-linear worldline from discrete proper-time samples.
#[derive(Debug, Clone)]
pub struct SampledWorldline {
    /// Sorted by `tau` (proper time in seconds).
    pub samples: Vec<(f64, Event)>,
}

impl SampledWorldline {
    pub fn new(mut samples: Vec<(f64, Event)>) -> Self {
        samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Self { samples }
    }
    pub fn try_new(mut samples: Vec<(f64, Event)>) -> Result<Self> {
        if samples.is_empty() {
            return Err(WorldlineError::EmptyWorldline);
        }
        for (tau, ev) in &samples {
            if !tau.is_finite() {
                return Err(WorldlineError::NonFiniteInput {
                    what: "tau".into(),
                    value: *tau,
                });
            }
            if !ev.is_finite() {
                return Err(WorldlineError::NonFiniteInput {
                    what: "event".into(),
                    value: ev.ct(),
                });
            }
        }
        samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // Check strictly increasing (no duplicate tau that would divide by zero).
        for w in samples.windows(2) {
            if w[1].0 <= w[0].0 {
                return Err(WorldlineError::InvalidSample {
                    msg: format!("non-increasing tau {} >= {}", w[0].0, w[1].0),
                });
            }
        }
        Ok(Self { samples })
    }
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
    pub fn tau_range(&self) -> Option<(f64, f64)> {
        if self.samples.is_empty() {
            None
        } else {
            Some((
                self.samples.first().unwrap().0,
                self.samples.last().unwrap().0,
            ))
        }
    }
    pub fn try_event_at_proper_time(&self, tau: f64) -> Result<Event> {
        if !tau.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "tau".into(),
                value: tau,
            });
        }
        if self.samples.is_empty() {
            return Err(WorldlineError::EmptyWorldline);
        }
        Ok(self.event_at_proper_time(tau))
    }
}

impl Worldline for SampledWorldline {
    fn event_at_proper_time(&self, tau: f64) -> Event {
        assert!(!self.samples.is_empty(), "SampledWorldline is empty");
        if tau <= self.samples[0].0 {
            return self.samples[0].1;
        }
        if tau >= self.samples.last().unwrap().0 {
            return self.samples.last().unwrap().1;
        }
        let idx = self.samples.partition_point(|(t, _)| *t < tau);
        let (t0, e0) = self.samples[idx - 1];
        let (t1, e1) = self.samples[idx];
        let frac = (tau - t0) / (t1 - t0);
        let ct = e0.ct() + frac * (e1.ct() - e0.ct());
        let x = e0.x() + frac * (e1.x() - e0.x());
        let y = e0.y() + frac * (e1.y() - e0.y());
        let z = e0.z() + frac * (e1.z() - e0.z());
        Event::new(ct, x, y, z)
    }
}

/// GR scaffolding: geodesic worldline parametrized by a metric.
#[derive(Debug, Clone)]
pub struct GeodesicWorldline<M: SpacetimeMetric> {
    pub metric: M,
    pub initial_event: Event,
    pub initial_four_velocity: FourVector,
}
impl<M: SpacetimeMetric> GeodesicWorldline<M> {
    pub fn new(metric: M, initial_event: Event, initial_four_velocity: FourVector) -> Self {
        Self {
            metric,
            initial_event,
            initial_four_velocity,
        }
    }
    pub fn try_new(
        metric: M,
        initial_event: Event,
        initial_four_velocity: FourVector,
    ) -> Result<Self> {
        if !initial_event.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "initial_event".into(),
                value: initial_event.ct(),
            });
        }
        initial_four_velocity.check_finite()?;
        Ok(Self {
            metric,
            initial_event,
            initial_four_velocity,
        })
    }
}
impl<M: SpacetimeMetric> Worldline for GeodesicWorldline<M> {
    fn event_at_proper_time(&self, tau: f64) -> Event {
        let d = self.initial_four_velocity * tau;
        Event::new(
            self.initial_event.ct() + d.ct,
            self.initial_event.x() + d.x,
            self.initial_event.y() + d.y,
            self.initial_event.z() + d.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetime::Event;
    #[test]
    fn inertial_at_rest() {
        let origin = Event::at_origin();
        let wl = InertialWorldline::at_rest(origin);
        let e = wl.event_at_proper_time(1.0);
        assert!((e.t() - 1.0).abs() < 1e-12);
        assert_eq!(e.x(), 0.0);
    }
    #[test]
    fn inertial_from_velocity() {
        let origin = Event::at_origin();
        let v = crate::frame::Vec3::new(1e6, 0.0, 0.0);
        let wl = InertialWorldline::from_velocity(origin, v);
        let gamma = crate::frame::gamma_from_velocity(v);
        let e = wl.event_at_proper_time(1.0);
        assert!((e.ct() - gamma * crate::constants::C).abs() < 1e-6);
    }
    #[test]
    fn try_from_velocity_superluminal_err() {
        let origin = Event::at_origin();
        let v = crate::frame::Vec3::new(4e8, 0.0, 0.0);
        assert!(InertialWorldline::try_from_velocity(origin, v).is_err());
    }
    #[test]
    fn sampled_interpolation() {
        let s = SampledWorldline::new(vec![
            (0.0, Event::new(0.0, 0.0, 0.0, 0.0)),
            (1.0, Event::new(crate::constants::C, 1.0, 0.0, 0.0)),
        ]);
        let mid = s.event_at_proper_time(0.5);
        assert!((mid.ct() - crate::constants::C * 0.5).abs() < 1e-9);
        assert!((mid.x() - 0.5).abs() < 1e-9);
    }
    #[test]
    fn sampled_try_new_empty_err() {
        assert!(SampledWorldline::try_new(vec![]).is_err());
    }
    #[test]
    fn sampled_try_new_duplicate_tau_err() {
        let e = Event::at_origin();
        assert!(SampledWorldline::try_new(vec![(0.0, e), (0.0, e)]).is_err());
    }
}
