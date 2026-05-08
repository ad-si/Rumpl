#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
  Idle,
  Attack,
  Decay,
  Sustain,
  Release,
}

#[derive(Clone, Copy, Debug)]
pub struct EnvSpec {
  pub attack: f32,
  pub decay: f32,
  pub sustain: f32, // 0..1
  pub release: f32,
}

impl EnvSpec {
  pub const fn ad(attack: f32, decay: f32) -> Self {
    Self {
      attack,
      decay,
      sustain: 0.0,
      release: 0.05,
    }
  }

  pub const fn adsr(
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
  ) -> Self {
    Self {
      attack,
      decay,
      sustain,
      release,
    }
  }
}

#[derive(Clone, Debug)]
pub struct Env {
  spec: EnvSpec,
  stage: Stage,
  value: f32,
  a_inc: f32,
  d_coef: f32,
  r_coef: f32,
}

impl Env {
  pub fn new(spec: EnvSpec, sr: f32) -> Self {
    let a_inc = if spec.attack > 0.0 {
      1.0 / (spec.attack * sr)
    } else {
      1.0
    };
    let d_coef = exp_coef(spec.decay, sr);
    let r_coef = exp_coef(spec.release, sr);
    Self {
      spec,
      stage: Stage::Idle,
      value: 0.0,
      a_inc,
      d_coef,
      r_coef,
    }
  }

  pub fn note_on(&mut self) {
    self.stage = Stage::Attack;
  }

  pub fn note_off(&mut self) {
    if self.stage != Stage::Idle {
      self.stage = Stage::Release;
    }
  }

  pub fn is_idle(&self) -> bool {
    self.stage == Stage::Idle
  }

  #[inline]
  pub fn tick(&mut self) -> f32 {
    match self.stage {
      Stage::Idle => 0.0,
      Stage::Attack => {
        self.value += self.a_inc;
        if self.value >= 1.0 {
          self.value = 1.0;
          self.stage = Stage::Decay;
        }
        self.value
      }
      Stage::Decay => {
        let target = self.spec.sustain;
        self.value = target + (self.value - target) * self.d_coef;
        if (self.value - target).abs() < 1e-4 {
          self.value = target;
          self.stage = if target <= 1e-4 {
            Stage::Idle
          } else {
            Stage::Sustain
          };
        }
        self.value
      }
      Stage::Sustain => self.value,
      Stage::Release => {
        self.value *= self.r_coef;
        if self.value < 1e-5 {
          self.value = 0.0;
          self.stage = Stage::Idle;
        }
        self.value
      }
    }
  }
}

/// Exponential decay coefficient that reaches ~1% in `time` seconds.
fn exp_coef(time: f32, sr: f32) -> f32 {
  if time <= 0.0 {
    return 0.0;
  }
  (-1.0 / (time * sr)).exp().powf(4.6) // 4.6 ≈ ln(100), so 1% in `time` s
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn attack_reaches_one_after_attack_time() {
    let sr = 44_100.0;
    let mut e = Env::new(EnvSpec::adsr(0.01, 0.1, 0.5, 0.1), sr);
    e.note_on();
    let n = (0.01 * sr) as usize;
    let mut peak = 0.0_f32;
    for _ in 0..n {
      peak = peak.max(e.tick());
    }
    assert!(peak >= 0.99, "peak attack value = {peak}");
  }

  #[test]
  fn release_decays_toward_zero() {
    let sr = 44_100.0;
    let mut e = Env::new(EnvSpec::adsr(0.001, 0.001, 1.0, 0.05), sr);
    e.note_on();
    for _ in 0..((0.005 * sr) as usize) {
      e.tick();
    }
    e.note_off();
    let mut v = 1.0;
    for _ in 0..((0.5 * sr) as usize) {
      v = e.tick();
    }
    assert!(v < 0.001, "release final value = {v}");
    assert!(e.is_idle());
  }
}
