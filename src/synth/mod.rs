pub mod channel;
pub mod drums;
pub mod env;
pub mod filter;
pub mod osc;
pub mod patches;
pub mod voice;

use channel::Channel;
use voice::Voice;

const MAX_VOICES: usize = 64;

pub struct Synth {
  pub sr: f32,
  pub channels: [Channel; 16],
  voices: Vec<Voice>,
  sample_clock: u64,
  pub master_gain: f32,
}

impl Synth {
  pub fn new(sr: f32) -> Self {
    let channels = std::array::from_fn(|i| Channel::new(i as u8));
    Self {
      sr,
      channels,
      voices: Vec::with_capacity(MAX_VOICES),
      sample_clock: 0,
      master_gain: 0.8,
    }
  }

  pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
    if velocity == 0 {
      self.note_off(channel, note);
      return;
    }
    let ch_idx = channel as usize;
    let ch = &self.channels[ch_idx];
    let vel = velocity_curve(velocity);
    let voice = if ch.is_drums {
      let dp = drums::drum_patch(note);
      // Choke other voices in the same group on this channel.
      if dp.group != 0 {
        for v in self.voices.iter_mut() {
          if v.channel == channel && v.drum_group() == dp.group {
            v.note_off(false);
          }
        }
      }
      Voice::new_drum(self.sr, channel, note, vel, &dp, self.sample_clock)
    } else {
      let p = patches::gm_patch(ch.program);
      Voice::new_tonal(
        self.sr,
        channel,
        note,
        vel,
        &p,
        ch.bend_semitones(),
        self.sample_clock,
      )
    };
    self.add_voice(voice);
  }

  pub fn note_off(&mut self, channel: u8, note: u8) {
    let sustain = self.channels[channel as usize].sustain;
    for v in self.voices.iter_mut() {
      if v.channel == channel
        && v.note == note
        && !v.released
        && !v.held_by_sustain
      {
        v.note_off(sustain);
      }
    }
  }

  pub fn all_notes_off(&mut self, channel: u8) {
    for v in self.voices.iter_mut() {
      if v.channel == channel {
        v.note_off(false);
      }
    }
  }

  pub fn all_sound_off(&mut self, channel: u8) {
    self.voices.retain(|v| v.channel != channel);
  }

  pub fn reset_controllers(&mut self, channel: u8) {
    let ch = &mut self.channels[channel as usize];
    ch.expression = 1.0;
    ch.mod_wheel = 0.0;
    ch.sustain = false;
    ch.pitch_bend = 0.0;
  }

  pub fn program_change(&mut self, channel: u8, program: u8) {
    self.channels[channel as usize].program = program;
  }

  pub fn pitch_bend(&mut self, channel: u8, bend: f32) {
    // The voice render loop reads `Channel::pitch_bend` each sample, so a
    // simple state update is sufficient.
    self.channels[channel as usize].pitch_bend = bend.clamp(-1.0, 1.0);
  }

  pub fn control_change(&mut self, channel: u8, cc: u8, value: u8) {
    let v = value as f32 / 127.0;
    let mut released_sustain = false;
    {
      let ch = &mut self.channels[channel as usize];
      match cc {
        1 => ch.mod_wheel = v,
        6 if ch.rpn_msb == 0 && ch.rpn_lsb == 0 => {
          ch.bend_range = value as f32;
        }
        7 => ch.volume = v,
        10 => ch.pan = v * 2.0 - 1.0,
        11 => ch.expression = v,
        64 => {
          let on = value >= 64;
          ch.sustain = on;
          if !on {
            released_sustain = true;
          }
        }
        100 => ch.rpn_lsb = value,
        101 => ch.rpn_msb = value,
        _ => {}
      }
    }
    if released_sustain {
      for v in self.voices.iter_mut() {
        if v.channel == channel {
          v.release_if_sustained();
        }
      }
      return;
    }
    match cc {
      120 => self.all_sound_off(channel),
      121 => self.reset_controllers(channel),
      123 => self.all_notes_off(channel),
      _ => {}
    }
  }

  /// Render `frames` stereo frames into `out` (length `frames * 2`). Buffer is overwritten.
  pub fn render(&mut self, out: &mut [f32]) {
    for s in out.iter_mut() {
      *s = 0.0;
    }
    let mg = self.master_gain;
    for v in self.voices.iter_mut() {
      let ch = &self.channels[v.channel as usize];
      v.render_add(out, ch);
    }
    for s in out.iter_mut() {
      *s *= mg;
      // Soft clip to avoid harsh distortion on dense passages.
      *s = soft_clip(*s);
    }
    let frames = out.len() / 2;
    self.sample_clock = self.sample_clock.wrapping_add(frames as u64);
    self.voices.retain(|v| !v.is_idle());
  }

  fn add_voice(&mut self, v: Voice) {
    if self.voices.len() >= MAX_VOICES {
      self.steal_voice();
    }
    self.voices.push(v);
  }

  fn steal_voice(&mut self) {
    // Prefer the oldest released voice; otherwise the oldest voice overall.
    let mut best: Option<usize> = None;
    let mut best_released = false;
    let mut best_age = u64::MAX;
    for (i, v) in self.voices.iter().enumerate() {
      if v.released && !best_released {
        best = Some(i);
        best_released = true;
        best_age = v.started_at;
      } else if v.released == best_released && v.started_at < best_age {
        best = Some(i);
        best_age = v.started_at;
      }
    }
    if let Some(i) = best {
      self.voices.swap_remove(i);
    }
  }
}

/// Standard MIDI velocity → linear-amplitude curve (squared).
fn velocity_curve(velocity: u8) -> f32 {
  let v = velocity as f32 / 127.0;
  v * v
}

#[inline]
fn soft_clip(x: f32) -> f32 {
  x.tanh()
}
