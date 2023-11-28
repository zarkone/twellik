use crate::point::Point;
use space::Metric;
pub struct Cosine;

fn dot_product_acc(acc: f64, (a, b): (&f64, &f64)) -> f64 {
    acc + a * b
}

fn dot_product(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    a.iter().zip(b.iter()).fold(0.0, dot_product_acc)
}

pub fn similarity(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert!(
        a.len() == b.len(),
        "cosine distance: vectors must have the same lenght"
    );

    let product = dot_product(&a, &b);

    let norm_a = dot_product(&a, &a).sqrt();
    let norm_b = dot_product(&b, &b).sqrt();

    product / (norm_a * norm_b)
}

// BTW, `space` crate must have the same version as in `hnsw` crate deps.
// Otherwise when you impl Metric it will be considered as different trate
// (although with the same name)
impl Metric<Point> for Cosine {
    type Unit = u64;
    fn distance(&self, a: &Point, b: &Point) -> u64 {
        let dist = 1.0 - similarity(&a.vector, &b.vector);
        dist.to_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_distance() {
        let v = [
            [6.6, 6.2, 1.0],
            [9.7, 9.9, 2.0],
            [8.0, 8.3, 2.0],
            [6.3, 5.4, 1.0],
            [1.3, 2.7, 0.0],
            [2.3, 3.1, 0.0],
            [6.6, 6.0, 1.0],
            [6.5, 6.4, 1.0],
            [6.3, 5.8, 1.0],
            [9.5, 9.9, 2.0],
            [8.9, 8.9, 2.0],
            [8.7, 9.5, 2.0],
            [2.5, 3.8, 0.0],
            [2.0, 3.1, 0.0],
            [1.3, 1.3, 0.0],
        ];

        let r1 = similarity(&v[14].to_vec(), &v[0].to_vec());
        assert_eq!(r1, 0.993472672904106);

        let r2 = similarity(&v[14].to_vec(), &v[1].to_vec());
        assert_eq!(r2, 0.9896970521184516);

        let r3 = similarity(&v[14].to_vec(), &v[4].to_vec());
        assert_eq!(r3, 0.9438583563660174);

        let v2 = [[6.6, 6.2], [9.7, 9.9]];
        let r4 = similarity(&v2[0].to_vec(), &v2[1].to_vec());
        assert_eq!(r4, 0.9991413385403556);
    }
}
