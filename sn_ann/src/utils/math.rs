#[inline]
pub fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    // with L2-normalized vectors, cosine == dot
    a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>()
}

#[inline]
fn l2norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

#[inline]
fn normalize(mut v: Vec<f32>) -> Vec<f32> {
    let n = l2norm(&v);
    if n > 0.0 {
        for x in &mut v {
            *x /= n;
        }
    }
    v
}
