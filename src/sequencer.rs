use crate::midi::{Event, Sequence};
use crate::synth::Synth;

/// Drives a `Synth` from a `Sequence` in fixed-size sample blocks.
pub struct Sequencer<'a> {
  seq: &'a Sequence,
  /// Index of the next pending event in `seq.events`.
  next: usize,
  /// Current sample-clock position.
  sample: u64,
  sr: f64,
  /// Total samples to render (last event time × sr + tail).
  total_samples: u64,
}

impl<'a> Sequencer<'a> {
  pub fn new(seq: &'a Sequence, sr: u32, tail_seconds: f32) -> Self {
    let total =
      ((seq.duration + tail_seconds as f64) * sr as f64).ceil() as u64;
    Self {
      seq,
      next: 0,
      sample: 0,
      sr: sr as f64,
      total_samples: total,
    }
  }

  pub fn total_samples(&self) -> u64 {
    self.total_samples
  }

  /// Render up to `frames` frames into `out` (interleaved stereo, length 2*frames),
  /// applying any events scheduled within that block. Returns the number of frames
  /// written (always == frames; partial-block handling is the caller's job).
  pub fn render_block(&mut self, synth: &mut Synth, out: &mut [f32]) -> usize {
    let frames = out.len() / 2;
    let block_end = self.sample + frames as u64;
    let mut cursor = 0usize; // frame within out

    while self.next < self.seq.events.len() {
      let ev = &self.seq.events[self.next];
      let ev_sample = (ev.time * self.sr) as u64;
      if ev_sample >= block_end {
        break;
      }
      let target_frame = if ev_sample <= self.sample {
        cursor
      } else {
        (ev_sample - self.sample) as usize
      }
      .min(frames);

      if target_frame > cursor {
        let slice = &mut out[cursor * 2..target_frame * 2];
        synth.render(slice);
        cursor = target_frame;
      }

      apply(synth, &ev.event);
      self.next += 1;
    }

    if cursor < frames {
      let slice = &mut out[cursor * 2..frames * 2];
      synth.render(slice);
    }

    self.sample = block_end;
    frames
  }
}

fn apply(synth: &mut Synth, ev: &Event) {
  match *ev {
    Event::NoteOn {
      channel,
      note,
      velocity,
    } => synth.note_on(channel, note, velocity),
    Event::NoteOff { channel, note } => synth.note_off(channel, note),
    Event::Cc { channel, cc, value } => {
      synth.control_change(channel, cc, value)
    }
    Event::Program { channel, program } => {
      synth.program_change(channel, program)
    }
    Event::PitchBend { channel, bend } => synth.pitch_bend(channel, bend),
    Event::EndOfTrack => {}
  }
}
