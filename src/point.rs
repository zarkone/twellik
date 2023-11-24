use rkyv::{Archive, Deserialize, Serialize};
use serde;
use std::collections::HashMap;

#[derive(Archive, Serialize, Deserialize, serde::Deserialize)]
#[archive(check_bytes)]
pub struct Point {
    /// TODO: id should be uuid or any
    pub id: u32,
    pub vector: Vec<f64>,
    pub payload: HashMap<String, String>,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
