//! Idiomatic demo phrases for auditioning a single GM instrument.
//!
//! Rather than playing one universal phrase for every program, each of the 16
//! GM families gets a phrase that suits the kind of music it is usually used
//! for — pianos arpeggiate a chord progression, basses walk a groove, strings
//! sing a legato line, brass stabs a fanfare, pads sustain, and so on. The
//! phrase is written out as a Standard MIDI File carrying the program change,
//! so it can be rendered or played like any other `.mid`.

use anyhow::{Context, Result};
use midly::num::{u4, u7, u15, u24, u28};
use midly::{
  Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent,
  TrackEventKind,
};

use crate::synth::patches::gm_program_name;

/// Ticks per quarter note for the generated files.
const TPQ: u32 = 480;

#[derive(Clone, Copy)]
struct Note {
  /// Onset in beats (quarter notes).
  start: f64,
  /// Duration in beats.
  dur: f64,
  key: u8,
  vel: u8,
}

/// Append a sequential melody: `keys` spaced `step` beats apart from `start`.
fn melody(
  out: &mut Vec<Note>,
  start: f64,
  step: f64,
  dur: f64,
  vel: u8,
  keys: &[u8],
) {
  for (i, &key) in keys.iter().enumerate() {
    out.push(Note {
      start: start + step * i as f64,
      dur,
      key,
      vel,
    });
  }
}

/// Append a simultaneous chord.
fn chord(out: &mut Vec<Note>, start: f64, dur: f64, vel: u8, keys: &[u8]) {
  for &key in keys {
    out.push(Note {
      start,
      dur,
      key,
      vel,
    });
  }
}

/// Tempo (BPM) and notes for a program's demo, chosen by GM family.
fn phrase(program: u8) -> (u32, Vec<Note>) {
  let mut n = Vec::new();
  let family = program / 8;
  let bpm = match family {
    0 => {
      // Piano — arpeggiated I–vi–IV–V progression with a sustained tonic.
      for (i, (bass, arp)) in [
        (48u8, [60u8, 64, 67, 72]),
        (45, [57, 60, 64, 69]),
        (41, [53, 57, 60, 65]),
        (43, [55, 59, 62, 67]),
      ]
      .iter()
      .enumerate()
      {
        let s = i as f64 * 2.0;
        chord(&mut n, s, 2.0, 70, &[*bass]);
        melody(&mut n, s, 0.5, 0.5, 88, arp);
      }
      chord(&mut n, 8.0, 2.5, 80, &[48, 60, 64, 67, 72]);
      104
    }
    1 => {
      // Chromatic percussion — a sparkly descending run and a turn.
      melody(
        &mut n,
        0.0,
        0.5,
        0.45,
        96,
        &[84, 83, 81, 79, 77, 76, 74, 72],
      );
      melody(&mut n, 4.0, 0.5, 0.45, 90, &[72, 76, 79, 84]);
      chord(&mut n, 6.0, 1.5, 88, &[72, 76, 79]);
      132
    }
    2 => {
      // Organ — sustained block chords I–IV–V–I with a top line.
      for (i, ch) in [
        [48u8, 55, 60, 64],
        [53, 57, 60, 65],
        [55, 59, 62, 67],
        [48, 55, 60, 64],
      ]
      .iter()
      .enumerate()
      {
        chord(&mut n, i as f64 * 2.0, 2.0, 82, ch);
      }
      melody(&mut n, 0.0, 1.0, 0.9, 96, &[72, 76, 77, 74, 79, 83, 79, 76]);
      96
    }
    3 => {
      // Guitar — fingerpicked arpeggios over a folk progression.
      for (i, (bass, arp)) in [
        (40u8, [52u8, 55, 59, 55]),
        (43, [52, 55, 60, 55]),
        (43, [50, 55, 59, 55]),
        (50, [57, 62, 66, 62]),
      ]
      .iter()
      .enumerate()
      {
        let s = i as f64 * 2.0;
        chord(&mut n, s, 2.0, 78, &[*bass]);
        melody(&mut n, s, 0.5, 0.5, 90, arp);
      }
      100
    }
    4 => {
      // Bass — a syncopated octave-and-fifth groove down low.
      melody(&mut n, 0.0, 0.5, 0.4, 96, &[36, 36, 43, 48, 36, 46, 43, 38]);
      melody(&mut n, 4.0, 0.5, 0.4, 96, &[41, 41, 48, 53, 41, 51, 48, 43]);
      112
    }
    5 => {
      // Strings — a singing, legato solo line.
      melody(&mut n, 0.0, 1.0, 1.1, 84, &[69, 72, 76, 74]);
      melody(&mut n, 4.0, 1.0, 1.1, 84, &[72, 69, 67, 69]);
      chord(&mut n, 8.0, 2.5, 80, &[72]);
      80
    }
    6 => {
      // Ensemble (strings/choir) — lush sustained four-part chords.
      for (i, ch) in [
        [48u8, 55, 64, 67],
        [45, 57, 64, 69],
        [41, 57, 60, 65],
        [43, 55, 59, 67],
      ]
      .iter()
      .enumerate()
      {
        chord(&mut n, i as f64 * 2.0, 2.0, 78, ch);
      }
      chord(&mut n, 8.0, 3.0, 78, &[48, 55, 64, 67]);
      72
    }
    7 => {
      // Brass — punchy fanfare stabs into a rising triad.
      chord(&mut n, 0.0, 0.4, 104, &[60, 64, 67]);
      chord(&mut n, 0.5, 0.4, 100, &[60, 64, 67]);
      chord(&mut n, 1.0, 0.4, 104, &[60, 64, 67]);
      melody(&mut n, 2.0, 0.5, 0.45, 104, &[67, 72, 76, 79]);
      chord(&mut n, 4.0, 2.0, 104, &[60, 64, 67, 72]);
      120
    }
    8 => {
      // Reed — a smooth, swung solo line.
      melody(
        &mut n,
        0.0,
        0.5,
        0.45,
        92,
        &[67, 69, 71, 72, 74, 72, 71, 67],
      );
      melody(
        &mut n,
        4.0,
        0.5,
        0.45,
        92,
        &[65, 67, 69, 71, 72, 69, 67, 65],
      );
      chord(&mut n, 8.0, 1.5, 88, &[64]);
      100
    }
    9 => {
      // Pipe — a light, airy melody up high.
      melody(
        &mut n,
        0.0,
        0.5,
        0.45,
        88,
        &[79, 81, 83, 84, 83, 81, 79, 76],
      );
      melody(&mut n, 4.0, 0.5, 0.45, 88, &[77, 79, 81, 79, 76, 72]);
      112
    }
    10 => {
      // Synth lead — a syncopated, repeated riff.
      melody(
        &mut n,
        0.0,
        0.5,
        0.4,
        100,
        &[72, 75, 79, 75, 72, 75, 79, 84],
      );
      melody(
        &mut n,
        4.0,
        0.5,
        0.4,
        100,
        &[82, 79, 75, 79, 75, 72, 75, 72],
      );
      124
    }
    11 => {
      // Synth pad — slow, sustained maj7 swells.
      chord(&mut n, 0.0, 4.0, 72, &[48, 55, 59, 64]);
      chord(&mut n, 4.0, 4.5, 72, &[41, 57, 60, 65]);
      60
    }
    12 => {
      // Synth effects — a low drone under scattered atmospheric tones.
      chord(&mut n, 0.0, 8.0, 64, &[36]);
      melody(&mut n, 0.0, 2.0, 1.8, 80, &[84, 79, 88, 83]);
      70
    }
    13 => {
      // Ethnic — a modal melody over a sustained drone.
      chord(&mut n, 0.0, 8.0, 70, &[50]);
      melody(
        &mut n,
        0.0,
        0.5,
        0.45,
        90,
        &[62, 64, 65, 67, 69, 67, 65, 64],
      );
      melody(&mut n, 4.0, 0.5, 0.45, 90, &[62, 64, 65, 64, 62, 62]);
      100
    }
    14 => {
      // Percussive (steel drum, bells, …) — a bright repeated riff.
      melody(
        &mut n,
        0.0,
        0.25,
        0.22,
        96,
        &[60, 60, 67, 60, 64, 60, 67, 64],
      );
      melody(
        &mut n,
        2.0,
        0.25,
        0.22,
        96,
        &[60, 60, 67, 72, 67, 64, 67, 60],
      );
      120
    }
    _ => {
      // Sound effects — non-pitched; just trigger the sound a few times.
      chord(&mut n, 0.0, 2.0, 100, &[60]);
      chord(&mut n, 2.0, 2.0, 100, &[67]);
      chord(&mut n, 4.0, 2.0, 100, &[72]);
      90
    }
  };
  (bpm, n)
}

/// Build the demo phrase for `program` as a Standard MIDI File and write it.
pub fn write_demo_midi(program: u8, path: &std::path::Path) -> Result<()> {
  let (bpm, notes) = phrase(program);
  let us_per_quarter = (60_000_000.0 / bpm as f64).round() as u32;
  let ch = u4::from(0);

  // Collect absolute-tick events, then convert to delta-time track events.
  // Sort key second component orders simultaneous events: setup (0) before
  // note-offs (1) before note-ons (2) before end-of-track (3).
  let mut evs: Vec<(u64, u8, TrackEventKind)> = vec![
    (
      0,
      0,
      TrackEventKind::Meta(MetaMessage::Tempo(u24::from(us_per_quarter))),
    ),
    (
      0,
      0,
      TrackEventKind::Midi {
        channel: ch,
        message: MidiMessage::ProgramChange {
          program: u7::from(program),
        },
      },
    ),
  ];

  for note in &notes {
    let on = (note.start * TPQ as f64).round() as u64;
    let off = ((note.start + note.dur) * TPQ as f64).round() as u64;
    evs.push((
      on,
      2,
      TrackEventKind::Midi {
        channel: ch,
        message: MidiMessage::NoteOn {
          key: u7::from(note.key),
          vel: u7::from(note.vel),
        },
      },
    ));
    evs.push((
      off,
      1,
      TrackEventKind::Midi {
        channel: ch,
        message: MidiMessage::NoteOff {
          key: u7::from(note.key),
          vel: u7::from(0),
        },
      },
    ));
  }

  let end = evs.iter().map(|e| e.0).max().unwrap_or(0) + TPQ as u64;
  evs.push((end, 3, TrackEventKind::Meta(MetaMessage::EndOfTrack)));
  evs.sort_by_key(|e| (e.0, e.1));

  let mut prev = 0u64;
  let mut track = Vec::with_capacity(evs.len());
  for (tick, _, kind) in evs {
    track.push(TrackEvent {
      delta: u28::from((tick - prev) as u32),
      kind,
    });
    prev = tick;
  }

  let smf = Smf {
    header: Header::new(
      Format::SingleTrack,
      Timing::Metrical(u15::from(TPQ as u16)),
    ),
    tracks: vec![track],
  };
  smf
    .save(path)
    .with_context(|| format!("writing {}", path.display()))
}

/// Slugged file name for a program's demo, e.g. `006-harpsichord.mid`.
pub fn demo_filename(program: u8) -> String {
  let slug = gm_program_name(program)
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() {
        c.to_ascii_lowercase()
      } else {
        ' '
      }
    })
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join("-");
  format!("{program:03}-{slug}.mid")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::midi;

  #[test]
  fn every_program_has_a_nonempty_phrase() {
    for program in 0..128u8 {
      let (bpm, notes) = phrase(program);
      assert!(bpm > 0);
      assert!(!notes.is_empty(), "program {program} has no notes");
      for note in &notes {
        assert!(note.dur > 0.0);
        assert!(note.key < 128 && note.vel < 128);
      }
    }
  }

  #[test]
  fn filenames_are_slugged_and_unique() {
    let mut names = std::collections::HashSet::new();
    for program in 0..128u8 {
      let name = demo_filename(program);
      assert!(name.ends_with(".mid"));
      assert!(!name.contains(' '));
      assert!(names.insert(name), "duplicate demo filename");
    }
    assert_eq!(demo_filename(6), "006-harpsichord.mid");
  }

  #[test]
  fn written_midi_parses_back_with_the_right_program() {
    let dir = std::env::temp_dir().join("rumpl-demo-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("040.mid");
    write_demo_midi(40, &path).unwrap();

    let info = midi::info(&path).unwrap();
    assert!(info.duration > 0.0);
    assert!(info.note_count > 0);
    assert_eq!(info.channels[0].program, 40);
  }
}
