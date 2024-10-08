use ndarray::{s, Array1};

use crate::stochastic::{noise::fgn::FGN, Sampling};

#[derive(Default)]
pub struct FJacobi {
  pub hurst: f64,
  pub alpha: f64,
  pub beta: f64,
  pub sigma: f64,
  pub n: usize,
  pub x0: Option<f64>,
  pub t: Option<f64>,
  pub m: Option<usize>,
  pub fgn: FGN,
}

impl FJacobi {
  #[must_use]
  pub fn new(params: &Self) -> Self {
    let fgn = FGN::new(params.hurst, params.n, params.t, params.m);

    Self {
      hurst: params.hurst,
      alpha: params.alpha,
      beta: params.beta,
      sigma: params.sigma,
      n: params.n,
      x0: params.x0,
      t: params.t,
      m: params.m,
      fgn,
    }
  }
}

impl Sampling<f64> for FJacobi {
  fn sample(&self) -> Array1<f64> {
    assert!(
      self.hurst > 0.0 && self.hurst < 1.0,
      "Hurst parameter must be in (0, 1)"
    );
    assert!(self.alpha > 0.0, "alpha must be positive");
    assert!(self.beta > 0.0, "beta must be positive");
    assert!(self.sigma > 0.0, "sigma must be positive");
    assert!(self.alpha < self.beta, "alpha must be less than beta");

    let dt = self.t.unwrap_or(1.0) / self.n as f64;
    let fgn = self.fgn.sample();

    let mut fjacobi = Array1::<f64>::zeros(self.n + 1);
    fjacobi[0] = self.x0.unwrap_or(0.0);

    for i in 1..=self.n {
      fjacobi[i] = match fjacobi[i - 1] {
        _ if fjacobi[i - 1] <= 0.0 && i > 0 => 0.0,
        _ if fjacobi[i - 1] >= 1.0 && i > 0 => 1.0,
        _ => {
          fjacobi[i - 1]
            + (self.alpha - self.beta * fjacobi[i - 1]) * dt
            + self.sigma * (fjacobi[i - 1] * (1.0 - fjacobi[i - 1])).sqrt() * fgn[i - 1]
        }
      }
    }

    fjacobi.slice(s![..self.n()]).to_owned()
  }

  fn n(&self) -> usize {
    self.n
  }

  fn m(&self) -> Option<usize> {
    self.m
  }
}
