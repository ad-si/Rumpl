use crate::synth::channel::Channel;
use crate::synth::drums::DrumPatch;
use crate::synth::env::Env;
use crate::synth::env::EnvSpec;
use crate::synth::filter::Svf;
use crate::synth::osc::{FmOp, Osc, Wave};
use crate::synth::patches::{LfoTarget, Patch};

pub fn midi_to_hz(note: f32) -> f32 {
  440.0 * (2.0_f32).powf((note - 69.0) / 12.0)
}

#[derive(Clone, Debug)]
struct OscState {
  osc: Osc,
  detune_cents: f32,
  mix: f32,
  octave: i32,
}

#[derive(Clone, Debug)]
struct LfoState {
  osc: Osc,
  depth: f32,
  target: LfoTarget,
  delay_samples: u32,
  age: u32,
}

#[derive(Clone, Debug)]
struct FilterState {
  svf: Svf,
  base_cutoff: f32,
  res: f32,
  env_amount: f32,
}

/// One string rank of a Karplus-Strong plucked-string voice: a delay line whose
/// length sets the pitch, fed back through a damping loop filter and a tuning
/// all-pass for fractional-sample pitch accuracy. The buffer is seeded at note
/// creation with a filtered noise burst — that burst *is* the pluck.
#[derive(Clone, Debug)]
struct KsString {
  buf: Vec<f32>,
  idx: usize,
  lp_x1: f32, // previous delay-line read (loop low-pass state)
  ap_x1: f32, // tuning all-pass input history
  ap_y1: f32, // tuning all-pass output history
  ap_c: f32,  // all-pass coefficient for the fractional delay
  decay: f32, // per-loop feedback gain (< 1)
  damp: f32,  // loop low-pass amount, 0..0.5
  mix: f32,
}

impl KsString {
  fn new(
    freq: f32,
    sr: f32,
    decay_time: f32,
    damp: f32,
    pluck_pos: f32,
    mix: f32,
    velocity: f32,
  ) -> Self {
    let damp = damp.clamp(0.0, 0.49);
    // Total loop delay must equal one period. The two-tap loop filter
    // contributes `damp` samples of delay; the rest is split into an integer
    // delay line plus a fractional all-pass.
    let period = (sr / freq.max(1.0)).max(2.0);
    let target = (period - damp).max(2.0);
    let mut n = target.floor() as usize;
    let mut frac = target - n as f32;
    // Keep the all-pass delay away from 0, where it is ill-conditioned.
    if frac < 0.1 && n > 2 {
      n -= 1;
      frac += 1.0;
    }
    let ap_c = (1.0 - frac) / (1.0 + frac);

    // Excitation: the initial string shape is a triangle peaking at the
    // plucking point. Its modal spectrum is sin(n·π·p)/n² — a strong
    // fundamental with a gentle roll-off and the pluck-position comb built in,
    // which is what a real pluck looks like. A little noise is mixed in for the
    // plectrum chiff, and the whole thing is low-passed by velocity so harder
    // plucks are brighter.
    const NOISE_AMT: f32 = 0.85;
    let p = pluck_pos.clamp(0.02, 0.5);
    let mut buf = vec![0.0f32; n];
    let mut rng = 0x9E37_79B9u32 ^ (n as u32).wrapping_mul(2_654_435_761);
    for (i, s) in buf.iter_mut().enumerate() {
      let x = i as f32 / n as f32;
      let tri = if x <= p { x / p } else { (1.0 - x) / (1.0 - p) };
      rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
      let white = ((rng >> 8) as f32 / 8_388_608.0) - 1.0;
      *s = tri + NOISE_AMT * white;
    }
    // Velocity-dependent brightness, then remove the triangle's DC offset so the
    // loop stays centred (a DC step would just thump and decay).
    let alpha = 0.72 + 0.28 * velocity.clamp(0.0, 1.0);
    let mut lp = 0.0f32;
    for s in buf.iter_mut() {
      lp += alpha * (*s - lp);
      *s = lp;
    }
    let mean = buf.iter().sum::<f32>() / n as f32;
    let mut peak = 1e-6f32;
    for s in buf.iter_mut() {
      *s -= mean;
      peak = peak.max(s.abs());
    }
    for s in buf.iter_mut() {
      *s /= peak;
    }

    Self {
      buf,
      idx: 0,
      lp_x1: 0.0,
      ap_x1: 0.0,
      ap_y1: 0.0,
      ap_c,
      decay: decay_gain(freq, decay_time),
      damp,
      mix,
    }
  }

  #[inline]
  fn tick(&mut self) -> f32 {
    let r = self.buf[self.idx];
    // Two-tap weighted average: the loop low-pass that damps high harmonics.
    let lp = (1.0 - self.damp) * r + self.damp * self.lp_x1;
    self.lp_x1 = r;
    // First-order all-pass for fractional-sample tuning.
    let ap = self.ap_c * lp + self.ap_x1 - self.ap_c * self.ap_y1;
    self.ap_x1 = lp;
    self.ap_y1 = ap;
    self.buf[self.idx] = ap * self.decay;
    self.idx += 1;
    if self.idx >= self.buf.len() {
      self.idx = 0;
    }
    r
  }
}

/// Per-loop feedback gain so the fundamental decays ~60 dB in `decay_time` s.
fn decay_gain(freq: f32, decay_time: f32) -> f32 {
  if decay_time <= 0.0 || freq <= 0.0 {
    return 0.9;
  }
  let passes = freq * decay_time;
  (0.001f32).powf(1.0 / passes).clamp(0.0, 0.999_95)
}

/// Higher strings ring shorter — halve the ring time for every two octaves
/// above middle C, and lengthen it for the bass.
fn key_decay_scale(note: u8) -> f32 {
  (2.0f32).powf(-(note as f32 - 60.0) / 24.0).clamp(0.3, 2.5)
}

#[derive(Clone, Debug)]
enum VoiceKind {
  Tonal {
    oscs: Vec<OscState>,
    fm: Option<FmOp>,
    filter: Option<FilterState>,
    lfo: Option<LfoState>,
  },
  Drum {
    tone: Option<Osc>,
    noise: Option<Osc>,
    tone_freq_base: f32,
    sweep_from: f32,
    sweep_inv_time: f32,
    sweep_t: f32,
    filter: Svf,
    tone_mix: f32,
    noise_mix: f32,
    group: u8,
  },
  Plucked {
    strings: Vec<KsString>,
    energy: f32, // peak follower for idle detection while a note is held
  },
}

#[derive(Clone, Debug)]
pub struct Voice {
  kind: VoiceKind,
  env: Env,
  sr: f32,
  pub channel: u8,
  pub note: u8,
  pub velocity: f32,
  pub started_at: u64,
  pub released: bool,
  pub held_by_sustain: bool,
  pub patch_gain: f32,
  mod_phase: f32,
}

impl Voice {
  pub fn new_tonal(
    sr: f32,
    channel: u8,
    note: u8,
    velocity: f32,
    patch: &Patch,
    bend_semitones: f32,
    started_at: u64,
  ) -> Self {
    let base_freq = midi_to_hz(note as f32 + bend_semitones);
    let oscs: Vec<OscState> = patch
      .oscs
      .iter()
      .map(|spec| {
        let mut o = Osc::new(spec.wave, sr);
        o.set_freq(
          base_freq
            * detune_factor(spec.detune_cents)
            * (2.0_f32).powi(spec.octave),
        );
        OscState {
          osc: o,
          detune_cents: spec.detune_cents,
          mix: spec.mix,
          octave: spec.octave,
        }
      })
      .collect();

    let fm = patch.fm.map(|fs| {
      let (ratio, index) = fs.parts();
      FmOp::new(ratio, index, sr)
    });

    let filter = patch.filter.map(|fs| {
      let kt = key_track_factor(note, fs.key_track);
      let cutoff = (fs.cutoff * kt).clamp(40.0, sr * 0.45);
      FilterState {
        svf: Svf::new(fs.mode, sr, cutoff, fs.res),
        base_cutoff: cutoff,
        res: fs.res,
        env_amount: fs.env_amount,
      }
    });

    let lfo = patch.lfo.map(|spec| {
      let mut o = Osc::new(Wave::Sine, sr);
      o.set_freq(spec.rate);
      LfoState {
        osc: o,
        depth: spec.depth,
        target: spec.target,
        delay_samples: (spec.delay * sr) as u32,
        age: 0,
      }
    });

    let mut env = Env::new(patch.env, sr);
    env.note_on();

    Self {
      kind: VoiceKind::Tonal {
        oscs,
        fm,
        filter,
        lfo,
      },
      env,
      sr,
      channel,
      note,
      velocity,
      started_at,
      released: false,
      held_by_sustain: false,
      patch_gain: patch.gain,
      mod_phase: 0.0,
    }
  }

  pub fn new_plucked(
    sr: f32,
    channel: u8,
    note: u8,
    velocity: f32,
    patch: &Patch,
    bend_semitones: f32,
    started_at: u64,
  ) -> Self {
    let spec = patch.pluck.expect("new_plucked requires a PluckSpec");
    let base_freq = midi_to_hz(note as f32 + bend_semitones);
    let ring = spec.decay * key_decay_scale(note);
    let strings = spec
      .strings
      .iter()
      .map(|st| {
        let f = base_freq
          * detune_factor(st.detune_cents)
          * (2.0_f32).powi(st.octave);
        KsString::new(f, sr, ring, spec.damp, spec.pluck_pos, st.mix, velocity)
      })
      .collect();

    // The pluck transient lives in the excitation, so the amplitude envelope is
    // just an instant gate that holds open and closes over `damper` seconds when
    // the key (and damper) drops. The string's own decay comes from the loop.
    let mut env =
      Env::new(EnvSpec::adsr(0.001, 0.1, 1.0, spec.damper.max(0.001)), sr);
    env.note_on();

    Self {
      kind: VoiceKind::Plucked {
        strings,
        energy: 1.0,
      },
      env,
      sr,
      channel,
      note,
      velocity,
      started_at,
      released: false,
      held_by_sustain: false,
      patch_gain: patch.gain,
      mod_phase: 0.0,
    }
  }

  pub fn new_drum(
    sr: f32,
    channel: u8,
    note: u8,
    velocity: f32,
    patch: &DrumPatch,
    started_at: u64,
  ) -> Self {
    let tone_osc = patch.tone.map(|t| {
      let mut o = Osc::new(t.wave, sr);
      o.set_freq(t.freq * patch.sweep.from);
      o
    });
    let noise = if patch.noise_mix > 0.0 {
      Some(Osc::new(Wave::Noise, sr))
    } else {
      None
    };
    let filter =
      Svf::new(patch.filter.mode, sr, patch.filter.cutoff, patch.filter.res);
    let mut env = Env::new(patch.env, sr);
    env.note_on();
    Self {
      kind: VoiceKind::Drum {
        tone: tone_osc,
        noise,
        tone_freq_base: patch.tone.map_or(0.0, |t| t.freq),
        sweep_from: patch.sweep.from,
        sweep_inv_time: if patch.sweep.time > 0.0 {
          1.0 / (patch.sweep.time * sr)
        } else {
          1e9
        },
        sweep_t: 0.0,
        filter,
        tone_mix: patch.tone.map_or(0.0, |t| t.mix),
        noise_mix: patch.noise_mix,
        group: patch.group,
      },
      env,
      sr,
      channel,
      note,
      velocity,
      started_at,
      released: true,
      held_by_sustain: false,
      patch_gain: patch.gain,
      mod_phase: 0.0,
    }
  }

  pub fn note_off(&mut self, sustain_on: bool) {
    if sustain_on {
      self.held_by_sustain = true;
      return;
    }
    self.env.note_off();
    self.released = true;
  }

  pub fn release_if_sustained(&mut self) {
    if self.held_by_sustain {
      self.held_by_sustain = false;
      self.env.note_off();
      self.released = true;
    }
  }

  pub fn is_idle(&self) -> bool {
    if let VoiceKind::Plucked { energy, .. } = &self.kind {
      // A held string is retired once it rings out; a released one once the
      // damper envelope has closed.
      return self.env.is_idle() || *energy < 1e-4;
    }
    self.env.is_idle()
  }

  pub fn drum_group(&self) -> u8 {
    if let VoiceKind::Drum { group, .. } = &self.kind {
      *group
    } else {
      0
    }
  }

  pub fn render_add(&mut self, out: &mut [f32], channel: &Channel) {
    let frames = out.len() / 2;
    let chan_gain = channel.gain();
    let total_gain = self.patch_gain * self.velocity * chan_gain;
    let (gl, gr) = equal_power_pan(channel.pan);

    match &mut self.kind {
      VoiceKind::Tonal {
        oscs,
        fm,
        filter,
        lfo,
      } => {
        let base_freq = midi_to_hz(self.note as f32 + channel.bend_semitones());
        let mod_wheel = channel.mod_wheel;
        for i in 0..frames {
          // LFO / mod-wheel modulations
          let (pitch_mod, filter_mod, amp_mod) = if let Some(l) = lfo.as_mut() {
            l.age = l.age.saturating_add(1);
            let ramp = if l.age < l.delay_samples {
              0.0
            } else {
              let r = (l.age - l.delay_samples) as f32 / (self.sr * 0.2);
              r.min(1.0)
            };
            let s = l.osc.tick() * ramp;
            match l.target {
              LfoTarget::Pitch => (s * (l.depth + mod_wheel * 0.05), 0.0, 0.0),
              LfoTarget::Filter => (0.0, s * l.depth, 0.0),
              LfoTarget::Amp => (0.0, 0.0, s * l.depth),
            }
          } else if mod_wheel > 0.0 {
            self.mod_phase += 5.0 / self.sr;
            if self.mod_phase >= 1.0 {
              self.mod_phase -= 1.0;
            }
            let pm =
              (self.mod_phase * std::f32::consts::TAU).sin() * mod_wheel * 0.03;
            (pm, 0.0, 0.0)
          } else {
            (0.0, 0.0, 0.0)
          };

          let pitch_factor = 2.0_f32.powf(pitch_mod);
          let mut sample = 0.0;
          if let Some(fop) = fm.as_mut() {
            sample = fop.tick(base_freq * pitch_factor);
          } else {
            let mut total_mix = 0.0;
            for o in oscs.iter_mut() {
              let f = base_freq
                * detune_factor(o.detune_cents)
                * (2.0_f32).powi(o.octave)
                * pitch_factor;
              o.osc.set_freq(f);
              sample += o.osc.tick() * o.mix;
              total_mix += o.mix;
            }
            if total_mix > 1e-6 {
              sample /= total_mix.max(1.0);
            }
          }

          let env = self.env.tick();
          let amp = env * total_gain * (1.0 + amp_mod);

          if let Some(fs) = filter.as_mut() {
            let env_offset = env * fs.env_amount * 4_000.0;
            let cutoff = (fs.base_cutoff + env_offset + filter_mod)
              .clamp(40.0, self.sr * 0.45);
            fs.svf.set(cutoff, fs.res);
            sample = fs.svf.tick(sample);
          }

          let s = sample * amp;
          out[i * 2] += s * gl;
          out[i * 2 + 1] += s * gr;
        }
      }
      VoiceKind::Drum {
        tone,
        noise,
        tone_freq_base,
        sweep_from,
        sweep_inv_time,
        sweep_t,
        filter,
        tone_mix,
        noise_mix,
        group: _,
      } => {
        let denom = (*tone_mix + *noise_mix).max(1e-6);
        for i in 0..frames {
          *sweep_t = (*sweep_t + *sweep_inv_time).min(1.0);
          let t = *sweep_t;
          let mult = *sweep_from + (1.0 - *sweep_from) * t;
          if let Some(o) = tone.as_mut() {
            o.set_freq(*tone_freq_base * mult);
          }
          let mut s = 0.0;
          if let Some(o) = tone.as_mut() {
            s += o.tick() * *tone_mix;
          }
          if let Some(o) = noise.as_mut() {
            s += o.tick() * *noise_mix;
          }
          s /= denom;
          s = filter.tick(s);
          let env = self.env.tick();
          let amp = env * total_gain;
          let v = s * amp;
          out[i * 2] += v * gl;
          out[i * 2 + 1] += v * gr;
        }
      }
      VoiceKind::Plucked { strings, energy } => {
        let total_mix: f32 = strings.iter().map(|s| s.mix).sum();
        let inv = if total_mix > 1e-6 {
          1.0 / total_mix
        } else {
          1.0
        };
        // Pitch is fixed by the delay-line length at note-on, so live pitch
        // bend is not tracked here (harpsichords cannot bend anyway).
        for i in 0..frames {
          let mut sample = 0.0;
          for s in strings.iter_mut() {
            sample += s.tick() * s.mix;
          }
          sample *= inv;
          *energy = (*energy * 0.9995).max(sample.abs());
          let amp = self.env.tick() * total_gain;
          let v = sample * amp;
          out[i * 2] += v * gl;
          out[i * 2 + 1] += v * gr;
        }
      }
    }
  }
}

fn detune_factor(cents: f32) -> f32 {
  (2.0_f32).powf(cents / 1200.0)
}

fn key_track_factor(note: u8, amount: f32) -> f32 {
  if amount <= 0.0 {
    return 1.0;
  }
  let octaves_from_c4 = (note as f32 - 60.0) / 12.0;
  (2.0_f32).powf(octaves_from_c4 * amount)
}

fn equal_power_pan(pan: f32) -> (f32, f32) {
  let p = pan.clamp(-1.0, 1.0);
  let theta = (p + 1.0) * 0.25 * std::f32::consts::PI;
  (theta.cos(), theta.sin())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn plucked_string_is_in_tune_and_decays() {
    let sr = 44_100.0;
    let mut s = KsString::new(440.0, sr, 2.0, 0.12, 0.13, 1.0, 1.0);
    let n = 8192;
    let buf: Vec<f32> = (0..n).map(|_| s.tick()).collect();

    // Estimate the fundamental by autocorrelation, skipping the attack.
    let expected = sr / 440.0; // ~100.2 samples
    let lo = (expected * 0.5) as usize;
    let hi = (expected * 1.5) as usize;
    let seg = &buf[1000..];
    let (mut best_lag, mut best) = (lo, f32::MIN);
    for lag in lo..hi {
      let acc: f32 = (0..seg.len() - lag).map(|i| seg[i] * seg[i + lag]).sum();
      if acc > best {
        best = acc;
        best_lag = lag;
      }
    }
    let f0 = sr / best_lag as f32;
    let cents = 1200.0 * (f0 / 440.0).log2();
    // Integer-lag autocorrelation only resolves to ~17 cents near this pitch;
    // the end-to-end FFT check is tighter. This guards gross mistuning.
    assert!(cents.abs() < 35.0, "f0 = {f0} Hz ({cents:+.1} cents)");

    let rms = |w: &[f32]| {
      (w.iter().map(|x| x * x).sum::<f32>() / w.len() as f32).sqrt()
    };
    let early = rms(&buf[..1000]);
    let late = rms(&buf[n - 1000..]);
    assert!(early > 1e-4, "string is silent (early rms = {early})");
    assert!(
      late < early,
      "string did not decay (early {early}, late {late})"
    );
  }
}
