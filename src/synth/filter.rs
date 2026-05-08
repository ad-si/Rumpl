use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterMode {
  Low,
  High,
  Band,
}

/// Topology-Preserving Transform state-variable filter (after Vadim Zavalishin).
#[derive(Clone, Debug)]
pub struct Svf {
  pub mode: FilterMode,
  pub sr: f32,
  cutoff: f32,
  res: f32,
  g: f32,
  k: f32,
  a1: f32,
  a2: f32,
  a3: f32,
  ic1eq: f32,
  ic2eq: f32,
}

impl Svf {
  pub fn new(mode: FilterMode, sr: f32, cutoff: f32, res: f32) -> Self {
    let mut s = Self {
      mode,
      sr,
      cutoff: 0.0,
      res: 0.0,
      g: 0.0,
      k: 0.0,
      a1: 0.0,
      a2: 0.0,
      a3: 0.0,
      ic1eq: 0.0,
      ic2eq: 0.0,
    };
    s.set(cutoff, res);
    s
  }

  pub fn set(&mut self, cutoff: f32, res: f32) {
    let c = cutoff.clamp(20.0, self.sr * 0.45);
    let r = res.clamp(0.5, 20.0);
    if (c - self.cutoff).abs() < 0.01 && (r - self.res).abs() < 0.001 {
      return;
    }
    self.cutoff = c;
    self.res = r;
    self.g = (PI * c / self.sr).tan();
    self.k = 1.0 / r;
    self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
    self.a2 = self.g * self.a1;
    self.a3 = self.g * self.a2;
  }

  #[inline]
  pub fn tick(&mut self, x: f32) -> f32 {
    let v3 = x - self.ic2eq;
    let v1 = self.a1 * self.ic1eq + self.a2 * v3;
    let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
    self.ic1eq = 2.0 * v1 - self.ic1eq;
    self.ic2eq = 2.0 * v2 - self.ic2eq;
    match self.mode {
      FilterMode::Low => v2,
      FilterMode::High => x - self.k * v1 - v2,
      FilterMode::Band => v1,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lowpass_attenuates_high_frequencies() {
    let sr = 44_100.0_f32;
    let mut f = Svf::new(FilterMode::Low, sr, 500.0, 0.7);
    let mut peak = 0.0_f32;
    // 5 kHz sine — well above cutoff
    let mut phase = 0.0_f32;
    let dt = 5_000.0 / sr;
    for _ in 0..(sr as usize) {
      let x = (phase * std::f32::consts::TAU).sin();
      phase = (phase + dt).fract();
      let y = f.tick(x);
      peak = peak.max(y.abs());
    }
    assert!(peak < 0.2, "lowpass peak too large: {peak}");
  }

  #[test]
  fn highpass_attenuates_low_frequencies() {
    let sr = 44_100.0_f32;
    let mut f = Svf::new(FilterMode::High, sr, 5_000.0, 0.7);
    let mut peak = 0.0_f32;
    let mut phase = 0.0_f32;
    let dt = 100.0 / sr;
    for _ in 0..(sr as usize) {
      let x = (phase * std::f32::consts::TAU).sin();
      phase = (phase + dt).fract();
      let y = f.tick(x);
      peak = peak.max(y.abs());
    }
    assert!(peak < 0.1, "highpass peak too large: {peak}");
  }
}
