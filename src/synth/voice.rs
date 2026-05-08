use crate::synth::channel::Channel;
use crate::synth::drums::DrumPatch;
use crate::synth::env::Env;
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
