use mlx_rs::Array;
use mlx_rs::nn::RmsNorm;

pub trait NormExt {
    fn update_weight(&mut self, x: &Array);
}

impl NormExt for RmsNorm {
    fn update_weight(&mut self, x: &Array) {
        self.weight.value = x.to_owned();
    }
}