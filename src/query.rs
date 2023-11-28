use crate::point::Point;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    pub vector: Vec<f64>,
    pub payload: HashMap<String, String>,
    pub k: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct QueryResult {
    pub point: Point,
    pub distance: f64,
}
