use ndarray::{s, Array1};

use crate::stochastic::{
  noise::fgn::FGN, process::cpoisson::CompoundPoisson, ProcessDistribution, Sampling, Sampling3D,
};

#[derive(Default)]
pub struct JumpFOU<D>
where
  D: ProcessDistribution,
{
  pub hurst: f64,
  pub mu: f64,
  pub sigma: f64,
  pub theta: f64,
  pub lambda: Option<f64>,
  pub n: usize,
  pub x0: Option<f64>,
  pub t: Option<f64>,
  pub m: Option<usize>,
  pub jump_distribution: D,
  pub fgn: FGN,
  pub cpoisson: CompoundPoisson<D>,
}

impl<D: ProcessDistribution> JumpFOU<D> {
  #[must_use]
  pub fn new(params: &JumpFOU<D>) -> Self {
    let fgn = FGN::new(params.hurst, params.n, params.t, params.m);

    let cpoisson = CompoundPoisson::new(&CompoundPoisson {
      n: None,
      lambda: params.lambda.unwrap(),
      t_max: Some(params.t.unwrap_or(1.0) / params.n as f64),
      distribution: params.jump_distribution,
      m: params.m,
      ..Default::default()
    });

    Self {
      hurst: params.hurst,
      mu: params.mu,
      sigma: params.sigma,
      theta: params.theta,
      lambda: params.lambda,
      n: params.n,
      x0: params.x0,
      t: params.t,
      m: params.m,
      jump_distribution: params.jump_distribution,
      fgn,
      cpoisson,
    }
  }
}

impl<D: ProcessDistribution> Sampling<f64> for JumpFOU<D> {
  fn sample(&self) -> Array1<f64> {
    assert!(
      self.hurst > 0.0 && self.hurst < 1.0,
      "Hurst parameter must be in (0, 1)"
    );

    let dt = self.t.unwrap_or(1.0) / self.n as f64;
    let fgn = self.fgn.sample();
    let mut jump_fou = Array1::<f64>::zeros(self.n + 1);
    jump_fou[0] = self.x0.unwrap_or(0.0);

    for i in 1..=self.n {
      let [.., jumps] = self.cpoisson.sample();

      jump_fou[i] = jump_fou[i - 1]
        + self.theta * (self.mu - jump_fou[i - 1]) * dt
        + self.sigma * fgn[i - 1]
        + jumps.sum();
    }

    jump_fou.slice(s![..self.n()]).to_owned()
  }

  fn n(&self) -> usize {
    self.n
  }

  fn m(&self) -> Option<usize> {
    self.m
  }
}
