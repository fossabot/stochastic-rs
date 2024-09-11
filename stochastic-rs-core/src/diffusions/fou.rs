use ndarray::Array1;

use crate::{noises::fgn::Fgn, Sampling};

pub struct Fou {
  pub hurst: f64,
  pub mu: f64,
  pub sigma: f64,
  pub theta: f64,
  pub n: usize,
  pub x0: Option<f64>,
  pub t: Option<f64>,
  pub m: Option<usize>,
  fgn: Fgn,
}

impl Fou {
  pub fn new(params: &Self) -> Self {
    let fgn = Fgn::new(params.hurst, params.n, params.t, None);

    Self {
      hurst: params.hurst,
      mu: params.mu,
      sigma: params.sigma,
      theta: params.theta,
      n: params.n,
      x0: params.x0,
      t: params.t,
      m: params.m,
      fgn,
    }
  }
}

impl Sampling<f64> for Fou {
  fn sample(&self) -> Array1<f64> {
    assert!(
      self.hurst > 0.0 && self.hurst < 1.0,
      "Hurst parameter must be in (0, 1)"
    );

    let dt = self.t.unwrap_or(1.0) / self.n as f64;
    let fgn = self.fgn.sample();

    let mut fou = Array1::<f64>::zeros(self.n + 1);
    fou[0] = self.x0.unwrap_or(100.0);

    for i in 1..(self.n + 1) {
      fou[i] = fou[i - 1] + self.theta * (self.mu - fou[i - 1]) * dt + self.sigma * fgn[i - 1]
    }

    fou
  }

  fn n(&self) -> usize {
    self.n
  }

  fn m(&self) -> Option<usize> {
    self.m
  }
}
