use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
  Sine,
  Triangle,
  Saw,
  Square,
  Noise,
}

#[derive(Clone, Debug)]
pub struct Osc {
  pub wave: Wave,
  pub phase: f32,
  pub freq: f32,
  pub sr: f32,
  rng: u32,
}

impl Osc {
  pub fn new(wave: Wave, sr: f32) -> Self {
    Self {
      wave,
      phase: 0.0,
      freq: 0.0,
      sr,
      rng: 0x9E37_79B9,
    }
  }

  pub fn set_freq(&mut self, hz: f32) {
    self.freq = hz.max(0.0);
  }

  #[inline]
  pub fn tick(&mut self) -> f32 {
    match self.wave {
      Wave::Noise => {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.rng >> 8) as f32 / 8_388_608.0) - 1.0
      }
      _ => {
        let dt = self.freq / self.sr;
        let p = self.phase;
        self.phase += dt;
        if self.phase >= 1.0 {
          self.phase -= 1.0;
        }
        match self.wave {
          Wave::Sine => (p * TAU).sin(),
          Wave::Triangle => {
            // Triangle from phase in [0,1)
            if p < 0.5 {
              -1.0 + 4.0 * p
            } else {
              3.0 - 4.0 * p
            }
          }
          Wave::Saw => {
            // Naive saw with PolyBLEP correction at the wrap.
            let mut v = 2.0 * p - 1.0;
            v -= poly_blep(p, dt);
            v
          }
          Wave::Square => {
            // Square = sign(saw); add PolyBLEP at both transitions.
            let mut v = if p < 0.5 { 1.0 } else { -1.0 };
            v += poly_blep(p, dt);
            v -= poly_blep((p + 0.5).fract(), dt);
            v
          }
          Wave::Noise => unreachable!(),
        }
      }
    }
  }
}

#[inline]
fn poly_blep(t: f32, dt: f32) -> f32 {
  if dt <= 0.0 {
    return 0.0;
  }
  if t < dt {
    let x = t / dt;
    2.0 * x - x * x - 1.0
  } else if t > 1.0 - dt {
    let x = (t - 1.0) / dt;
    x * x + 2.0 * x + 1.0
  } else {
    0.0
  }
}

/// Two-operator FM operator: carrier modulated by another sine.
#[derive(Clone, Debug)]
pub struct FmOp {
  pub carrier_phase: f32,
  pub mod_phase: f32,
  pub ratio: f32, // modulator / carrier
  pub index: f32, // modulation depth in radians
  pub sr: f32,
}

impl FmOp {
  pub fn new(ratio: f32, index: f32, sr: f32) -> Self {
    Self {
      carrier_phase: 0.0,
      mod_phase: 0.0,
      ratio,
      index,
      sr,
    }
  }

  #[inline]
  pub fn tick(&mut self, freq: f32) -> f32 {
    let cf = freq / self.sr;
    let mf = (freq * self.ratio) / self.sr;
    self.mod_phase += mf;
    if self.mod_phase >= 1.0 {
      self.mod_phase -= 1.0;
    }
    let m = (self.mod_phase * TAU).sin() * self.index;
    self.carrier_phase += cf;
    if self.carrier_phase >= 1.0 {
      self.carrier_phase -= 1.0;
    }
    (self.carrier_phase * TAU + m).sin()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sine_rms_close_to_root_two_over_two() {
    let sr = 44_100.0;
    let mut o = Osc::new(Wave::Sine, sr);
    o.set_freq(440.0);
    let n = sr as usize;
    let mut ss = 0.0;
    for _ in 0..n {
      let v = o.tick();
      ss += v * v;
    }
    let rms = (ss / n as f32).sqrt();
    assert!(
      (rms - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.01,
      "rms = {rms}"
    );
  }

  #[test]
  fn saw_in_range() {
    let mut o = Osc::new(Wave::Saw, 44_100.0);
    o.set_freq(220.0);
    let mut peak = 0.0_f32;
    for _ in 0..44_100 {
      let v = o.tick();
      peak = peak.max(v.abs());
    }
    assert!(peak <= 1.05, "saw exceeded expected range: {peak}");
    assert!(peak > 0.9, "saw amplitude too low: {peak}");
  }

  #[test]
  fn noise_is_unbiased() {
    let mut o = Osc::new(Wave::Noise, 44_100.0);
    let mut sum = 0.0;
    let n = 100_000;
    for _ in 0..n {
      sum += o.tick();
    }
    let mean = sum / n as f32;
    assert!(mean.abs() < 0.05, "noise mean = {mean}");
  }
}
