use rkyv::Archive;
use serde;
use std::collections::HashMap;

#[derive(
    Archive, rkyv::Serialize, rkyv::Deserialize, serde::Deserialize, serde::Serialize, Clone, Debug,
)]
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
