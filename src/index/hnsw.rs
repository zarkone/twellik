use crate::dist::Cosine;
use crate::index::Index;
use crate::log;
use crate::point::Point;
use crate::query::{Query, QueryResult};

use std::collections::HashMap;
use std::f64;

use hnsw::{Hnsw, Searcher};
use rand_pcg::Pcg64;
use space::Neighbor;

pub struct HnswIndex {
    index: Hnsw<Cosine, Point, Pcg64, 12, 24>,
}

impl HnswIndex {
    pub fn default() -> Self {
        HnswIndex {
            index: Hnsw::new(Cosine),
        }
    }
}

impl Index for HnswIndex {
    fn insert(&mut self, point: Point) {
        let mut searcher: Searcher<u64> = Searcher::default();
        self.index.insert(point, &mut searcher);
    }

    fn scroll(&self, query: &Query) -> Vec<QueryResult> {
        log::debug(format!("hnsw scroll {:?}", query).as_str());
        log::debug(format!("hnsw scroll: index len {}", &self.index.len()).as_str());
        // TODO: for some reason, searcher has to be mutable??
        // just creating it here to avoid the chain of mutable borrows
        let mut searcher: Searcher<u64> = Searcher::default();

        // instead of returning a vector of neighbors, hnsw.nearest takes a &mut neighbors in params.. :<
        // and fills all of them
        let mut neighbors: Vec<Neighbor<u64>> = Vec::new();
        for _ in 0..query.k {
            neighbors.push(Neighbor {
                distance: 0,
                index: 0,
            })
        }
        // see https://docs.rs/hnsw/0.11.0/hnsw/struct.Hnsw.html#method.nearest
        let ef = query.k * 2;
        let query_point = Point {
            id: 0,
            vector: query.vector.clone(),
            payload: query.payload.clone(),
        };
        self.index
            .nearest(&query_point, ef, &mut searcher, &mut neighbors);

        let mut result: Vec<QueryResult> = vec![];

        for neighbor in &neighbors {
            let point = self.index.feature(neighbor.index);

            result.push(QueryResult {
                point: point.clone(),
                distance: f64::from_bits(neighbor.distance),
            })
        }

        log::debug(
            format!(
                "hnsw scroll result payload: {:?}",
                &result
                    .iter()
                    .map(|r| r.point.payload.clone())
                    .collect::<Vec<HashMap<String, String>>>()
            )
            .as_str(),
        );
        result
    }
}

/// Checks if all fields of `query_fields` are eq to those in `item`
fn match_payload(item: &HashMap<String, String>, query_fields: &HashMap<String, String>) -> bool {
    if query_fields.is_empty() {
        return true;
    }

    for (key, val) in query_fields {
        let item_val = item.get(key);
        if let Some(found_key) = item_val {
            if found_key.eq(val) {
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::match_payload;
    use std::collections::HashMap;

    #[test]
    fn match_payload_test_happy() {
        let item = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
            ("c".to_string(), "three".to_string()),
        ]);

        let query_fields = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
            ("c".to_string(), "three".to_string()),
        ]);

        let result = match_payload(&item, &query_fields);

        assert!(result);
    }

    #[test]
    fn match_payload_test_two() {
        let item = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
            ("c".to_string(), "three".to_string()),
        ]);

        let query_fields = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "one".to_string()),
        ]);

        let result = match_payload(&item, &query_fields);

        assert!(!result);
    }
}
