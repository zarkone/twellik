use crate::index::Index;
use crate::point::Point;
use std::collections::HashMap;

pub type PointId = u32;

pub struct Collection {
    pub name: String,

    // TODO: workaround to fix duplication
    // TODO: workaround until I figured how to Archive HNSW
    pub points: HashMap<PointId, Point>,

    pub index: Box<dyn Index>,
}
