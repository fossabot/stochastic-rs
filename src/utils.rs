#[repr(C)]
#[derive(Clone, Copy)]
pub enum NoiseGenerationMethod {
  Cholesky,
  Fft,
}

pub trait Generator: Sync + Send {
  fn sample(&self) -> Vec<f64>;
  fn sample_par(&self) -> Vec<Vec<f64>>;
}
