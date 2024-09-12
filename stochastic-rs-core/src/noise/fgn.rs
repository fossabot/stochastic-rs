use std::sync::{Arc, Mutex};

use ndarray::parallel::prelude::*;
use ndarray::{concatenate, prelude::*};
use ndarray_rand::rand_distr::StandardNormal;
use ndarray_rand::RandomExt;
use ndrustfft::{ndfft, FftHandler};
use num_complex::{Complex, ComplexDistribution};

use crate::Sampling;

pub struct Fgn {
  pub hurst: f64,
  pub n: usize,
  pub t: Option<f64>,
  pub m: Option<usize>,
  pub offset: usize,
  pub sqrt_eigenvalues: Arc<Array1<Complex<f64>>>,
  pub fft_handler: Arc<FftHandler<f64>>,
}

impl Default for Fgn {
  fn default() -> Self {
    Self::new(0.7, 1024, None, None)
  }
}

impl Fgn {
  #[must_use]
  pub fn new(hurst: f64, n: usize, t: Option<f64>, m: Option<usize>) -> Self {
    if !(0.0..=1.0).contains(&hurst) {
      panic!("Hurst parameter must be between 0 and 1");
    }
    let n_ = n.next_power_of_two();
    let offset = n_ - n;
    let n = n_;
    let mut r = Array1::linspace(0.0, n as f64, n + 1);
    r.mapv_inplace(|x| {
      if x == 0.0 {
        1.0
      } else {
        0.5
          * ((x + 1.0).powf(2.0 * hurst) - 2.0 * x.powf(2.0 * hurst) + (x - 1.0).powf(2.0 * hurst))
      }
    });
    let r = concatenate(
      Axis(0),
      #[allow(clippy::reversed_empty_ranges)]
      &[r.view(), r.slice(s![..;-1]).slice(s![1..-1]).view()],
    )
    .unwrap();
    let data = r.mapv(|v| Complex::new(v, 0.0));
    let r_fft = FftHandler::new(r.len());
    let mut sqrt_eigenvalues = Array1::<Complex<f64>>::zeros(r.len());
    ndfft(&data, &mut sqrt_eigenvalues, &r_fft, 0);
    sqrt_eigenvalues.mapv_inplace(|x| Complex::new((x.re / (2.0 * n as f64)).sqrt(), x.im));

    Self {
      hurst,
      n,
      offset,
      t,
      sqrt_eigenvalues: Arc::new(sqrt_eigenvalues),
      m,
      fft_handler: Arc::new(FftHandler::new(2 * n)),
    }
  }
}

impl Sampling<f64> for Fgn {
  fn sample(&self) -> Array1<f64> {
    let num_threads = rayon::current_num_threads();
    let chunk_size = (2 * self.n) / num_threads;
    let rnd = Arc::new(Mutex::new(Array1::<Complex<f64>>::zeros(2 * self.n)));

    (0..num_threads).into_par_iter().for_each(|i| {
      let chunk = Array1::<Complex<f64>>::random(
        chunk_size,
        ComplexDistribution::new(StandardNormal, StandardNormal),
      );

      let mut result_lock = rnd.lock().unwrap();
      result_lock
        .slice_mut(s![i * chunk_size..(i + 1) * chunk_size])
        .assign(&chunk);
    });

    let fgn = &*self.sqrt_eigenvalues * &*rnd.lock().unwrap();
    let mut fgn_fft = Array1::<Complex<f64>>::zeros(2 * self.n);
    ndfft(&fgn, &mut fgn_fft, &*self.fft_handler, 0);
    let scale = (self.n as f64).powf(-self.hurst) * self.t.unwrap_or(1.0).powf(self.hurst);
    let fgn = fgn_fft
      .slice(s![1..self.n - self.offset + 1])
      .mapv(|x: Complex<f64>| x.re * scale);
    fgn
  }

  fn n(&self) -> usize {
    self.n
  }

  fn m(&self) -> Option<usize> {
    self.m
  }
}

#[cfg(test)]
mod tests {
  use plotly::{common::Line, Plot, Scatter};

  use super::*;

  #[test]
  fn plot() {
    let fgn = Fgn::new(0.7, 1024, Some(1.0), Some(1));
    let mut plot = Plot::new();
    let d = fgn.sample_par();
    for data in d.axis_iter(Axis(0)) {
      let trace = Scatter::new((0..data.len()).collect::<Vec<_>>(), data.to_vec())
        .mode(plotly::common::Mode::Lines)
        .line(
          Line::new()
            .color("orange")
            .shape(plotly::common::LineShape::Linear),
        )
        .name("Fgn");
      plot.add_trace(trace);
    }
    plot.show();
  }
}
