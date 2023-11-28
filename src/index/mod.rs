mod hnsw;
pub use crate::index::hnsw::HnswIndex;
use crate::point::Point;
use crate::query::{Query, QueryResult};

pub trait Index {
    fn insert(&mut self, point: Point);
    fn scroll(&self, query: &Query) -> Vec<QueryResult>;
}
