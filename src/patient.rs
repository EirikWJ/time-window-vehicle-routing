use serde::Deserialize;

use crate::utils::{Coordinate, deserialize_u32_from_number};

#[derive(Debug, Deserialize)]
pub struct Patient {
    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub demand: u32,

    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub start_time: u32,

    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub end_time: u32,

    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub care_time: u32,

    #[serde(flatten)]
    pub coord: Coordinate,
}