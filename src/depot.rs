use serde::Deserialize;

use crate::utils::{deserialize_i32_from_number, deserialize_u32_from_number};

#[derive(Debug, Deserialize)]
pub struct Depot {
    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub return_time: u32,

    #[serde(deserialize_with = "deserialize_i32_from_number")]
    pub x_coord: i32,

    #[serde(deserialize_with = "deserialize_i32_from_number")]
    pub y_coord: i32,
}
