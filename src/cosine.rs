/// TODO
/// something copied from internet:
/// already regret it since just to make it work properly
/// I've changed half of the code..
pub fn distance(vec1: &Vec<f64>, vec2: &Vec<f64>) -> f64 {
    let dot_product = dot_product(vec1, vec2);
    let root_sum_square1 = root_sum_square(vec1);
    let root_sum_square2 = root_sum_square(vec2);
    return dot_product as f64 / (root_sum_square1 * root_sum_square2);
}

fn root_sum_square(vec: &Vec<f64>) -> f64 {
    let mut sum_square: f64 = 0.0;
    for i in 0..vec.len() {
        sum_square += vec[i] * vec[i];
    }
    (sum_square as f64).sqrt()
}

fn dot_product(vec1: &Vec<f64>, vec2: &Vec<f64>) -> f64 {
    assert!(
        vec1.len() == vec2.len(),
        "vectors must have the same length"
    );

    let mut dot_product: f64 = 0.0;
    for i in 0..vec1.len() {
        dot_product += vec1[i] * vec2[i];
    }
    dot_product
}
