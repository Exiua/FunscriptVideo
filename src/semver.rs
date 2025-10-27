use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemVerError {
    #[error("Invalid version format")]
    InvalidFormat,
    #[error("Invalid number in version: {0}")]
    InvalidNumber(String),
}

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch }
    }

    pub fn parse(version_str: &str) -> Result<Self, SemVerError> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(SemVerError::InvalidFormat);
        }

        let major = parts[0].parse::<u32>().map_err(|_| SemVerError::InvalidNumber(parts[0].into()))?;
        let minor = parts[1].parse::<u32>().map_err(|_| SemVerError::InvalidNumber(parts[1].into()))?;
        let patch = parts[2].parse::<u32>().map_err(|_| SemVerError::InvalidNumber(parts[2].into()))?;

        Ok(Version::new(major, minor, patch))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.major != other.major {
            return self.major.cmp(&other.major);
        }
        else if self.minor != other.minor {
            return self.minor.cmp(&other.minor);
        }
        else {
            return self.patch.cmp(&other.patch);
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let version_str = format!("{}.{}.{}", self.major, self.minor, self.patch);
        serializer.serialize_str(&version_str)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let version_str = String::deserialize(deserializer)?;
        Version::parse(&version_str).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_parse_invalid_format() {
        let err = Version::parse("1.2").unwrap_err();
        match err {
            SemVerError::InvalidFormat => (),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_version_parse_invalid_number() {
        let err = Version::parse("1.a.3").unwrap_err();
        match err {
            SemVerError::InvalidNumber(num) => assert_eq!(num, "a"),
            _ => panic!("Expected InvalidNumber error"),
        }
    }

    #[test]
    fn test_version_display() {
        let version = Version { major: 1, minor: 2, patch: 3 };
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_serialize_deserialize() {
        let version = Version { major: 1, minor: 2, patch: 3 };
        let serialized = serde_json::to_string(&version).unwrap();
        assert_eq!(serialized, "\"1.2.3\"");

        let deserialized: Version = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.major, 1);
        assert_eq!(deserialized.minor, 2);
        assert_eq!(deserialized.patch, 3);
    }

    #[test]
    fn test_version_deserialize_invalid() {
        let serialized = "\"1.2\"";
        let err: serde_json::Error = serde_json::from_str::<Version>(serialized).unwrap_err();
        assert!(err.to_string().contains("Invalid version format"));

        let serialized = "\"1.a.3\"";
        let err: serde_json::Error = serde_json::from_str::<Version>(serialized).unwrap_err();
        assert!(err.to_string().contains("Invalid number in version: a"));
    }
}