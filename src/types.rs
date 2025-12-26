use std::num::ParseFloatError;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(try_from = "&str")]
pub struct Price(pub f64);

impl TryFrom<&str> for Price {
  type Error = ParseFloatError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Ok(Price(f64::from_str(value)?))
  }
}

impl Ord for Price {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.partial_cmp(&other.0).unwrap()
  }
}

impl Eq for Price {}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "&str")]
pub struct Volume(pub usize);

impl TryFrom<&str> for Volume {
  type Error = ParseFloatError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Ok(Volume(f64::from_str(value)? as usize))
  }
}
