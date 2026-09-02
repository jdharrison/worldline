//! Structured errors for `worldline` — never panic in library code.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldlineError {
    Superluminal {
        beta: f64,
        velocity_norm: f64,
    },
    InvalidGamma {
        gamma: f64,
    },
    InvalidMass {
        mass: f64,
    },
    InvalidTimeScale {
        msg: String,
    },
    OutOfBounds {
        what: String,
        value: f64,
        min: f64,
        max: f64,
    },
    EmptyWorldline,
    InvalidSample {
        msg: String,
    },
    NonFiniteInput {
        what: String,
        value: f64,
    },
    MetricError(String),
    TimeError(String),
}

impl fmt::Display for WorldlineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Superluminal {
                beta,
                velocity_norm,
            } => write!(
                f,
                "superluminal velocity: |v|={} m/s beta={} >=1 (c={})",
                velocity_norm,
                beta,
                crate::constants::C
            ),
            Self::InvalidGamma { gamma } => write!(f, "invalid gamma: {}", gamma),
            Self::InvalidMass { mass } => write!(f, "invalid mass: {}", mass),
            Self::InvalidTimeScale { msg } => write!(f, "invalid time scale: {}", msg),
            Self::OutOfBounds {
                what,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "{} out of bounds: {} not in [{},{}]",
                    what, value, min, max
                )
            }
            Self::EmptyWorldline => write!(f, "worldline has no samples"),
            Self::InvalidSample { msg } => write!(f, "invalid sample: {}", msg),
            Self::NonFiniteInput { what, value } => {
                write!(f, "non-finite input {}: {}", what, value)
            }
            Self::MetricError(s) => write!(f, "metric error: {}", s),
            Self::TimeError(s) => write!(f, "time error: {}", s),
        }
    }
}

impl std::error::Error for WorldlineError {}

pub type Result<T> = std::result::Result<T, WorldlineError>;

pub fn check_finite(what: &str, v: f64) -> Result<()> {
    if !v.is_finite() {
        return Err(WorldlineError::NonFiniteInput {
            what: what.to_string(),
            value: v,
        });
    }
    Ok(())
}

pub fn check_beta(beta: f64) -> Result<()> {
    check_finite("beta", beta)?;
    if beta.abs() >= 1.0 {
        return Err(WorldlineError::Superluminal {
            beta,
            velocity_norm: beta * crate::constants::C,
        });
    }
    if beta.abs() > 1.0 - 1e-15 && beta.abs() < 1.0 {
        // Extremely close to c — gamma would overflow; caller should handle.
    }
    Ok(())
}
