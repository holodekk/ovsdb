use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Represents the low-level scalar type contained in an OVSDB [Column][super::Column].
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Atomic {
    /// [bool] value
    Boolean,
    /// [i64] value
    Integer,
    /// [f64] value
    Real,
    /// [String] value
    #[default]
    String,
    /// Unique identifier conforming to the UUID v4 specification
    Uuid,
}

impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "boolean"),
            Self::Integer => write!(f, "integer"),
            Self::Real => write!(f, "real"),
            Self::String => write!(f, "string"),
            Self::Uuid => write!(f, "uuid"),
        }
    }
}

impl FromStr for Atomic {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(value: &str) -> std::result::Result<Atomic, Self::Err> {
        match value {
            "boolean" | "Boolean" => Ok(Self::Boolean),
            "integer" | "Integer" => Ok(Self::Integer),
            "real" | "Real" => Ok(Self::Real),
            "string" | "String" => Ok(Self::String),
            "uuid" | "Uuid" => Ok(Self::Uuid),
            _ => Err(format!("Not a valid atomic value: {}", value).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_deserialize_boolean() {
        let data = r#""boolean""#;
        let a: Atomic = serde_json::from_str(data).expect("bool from str");
        assert_eq!(a, Atomic::Boolean);
    }
}
