use anyhow::{Context, Result, anyhow};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

#[derive(Clone, Debug)]
pub enum Event {
  NoteOn { channel: u8, note: u8, velocity: u8 },
  NoteOff { channel: u8, note: u8 },
  Cc { channel: u8, cc: u8, value: u8 },
  Program { channel: u8, program: u8 },
  PitchBend { channel: u8, bend: f32 },
  EndOfTrack,
}

#[derive(Clone, Debug)]
pub struct TimedEvent {
  /// Absolute time in seconds from the start of the song.
  pub time: f64,
  pub event: Event,
}

pub struct Sequence {
  pub events: Vec<TimedEvent>,
  pub duration: f64,
}

pub fn load(path: &std::path::Path) -> Result<Sequence> {
  let bytes = std::fs::read(path)
    .with_context(|| format!("reading {}", path.display()))?;
  parse(&bytes)
}

/// Per-channel usage summary derived from a parsed file.
#[derive(Clone, Debug)]
pub struct ChannelInfo {
  /// 0-based MIDI channel.
  pub channel: u8,
  /// Channel 9 is the GM drum channel.
  pub is_drums: bool,
  /// Last program selected on the channel (0 if none was set).
  pub program: u8,
  /// Number of note-on events (velocity > 0) on the channel.
  pub note_count: usize,
}

/// File-level metadata for the `info` subcommand.
#[derive(Clone, Debug)]
pub struct FileInfo {
  /// SMF format: 0 (single track), 1 (parallel), or 2 (sequential).
  pub format: u8,
  /// Number of tracks in the file.
  pub track_count: usize,
  /// Pulses per quarter note, or `None` for SMPTE-timed files.
  pub ppq: Option<u16>,
  /// Total length in seconds.
  pub duration: f64,
  /// Distinct tempos in BPM, in order of first appearance.
  pub tempos_bpm: Vec<f64>,
  /// Total note-on events (velocity > 0) across all channels.
  pub note_count: usize,
  /// Channels that carry at least one event, sorted by channel number.
  pub channels: Vec<ChannelInfo>,
}

pub fn info(path: &std::path::Path) -> Result<FileInfo> {
  let bytes = std::fs::read(path)
    .with_context(|| format!("reading {}", path.display()))?;
  analyze(&bytes)
}

pub fn analyze(bytes: &[u8]) -> Result<FileInfo> {
  let smf = Smf::parse(bytes).map_err(|e| anyhow!("parse MIDI: {e}"))?;
  let seq = parse(bytes)?;

  let (format, ppq) = match smf.header.timing {
    Timing::Metrical(t) => {
      let fmt = match smf.header.format {
        midly::Format::SingleTrack => 0,
        midly::Format::Parallel => 1,
        midly::Format::Sequential => 2,
      };
      (fmt, Some(t.as_int()))
    }
    Timing::Timecode(..) => {
      let fmt = match smf.header.format {
        midly::Format::SingleTrack => 0,
        midly::Format::Parallel => 1,
        midly::Format::Sequential => 2,
      };
      (fmt, None)
    }
  };

  // Distinct tempos in order of first appearance across the merged timeline.
  let mut tempos_bpm: Vec<f64> = Vec::new();
  for track in smf.tracks.iter() {
    for ev in track.iter() {
      if let TrackEventKind::Meta(MetaMessage::Tempo(uspq)) = ev.kind {
        let bpm = 60_000_000.0 / uspq.as_int() as f64;
        if !tempos_bpm.iter().any(|&b| (b - bpm).abs() < 1e-3) {
          tempos_bpm.push(bpm);
        }
      }
    }
  }
  if tempos_bpm.is_empty() {
    tempos_bpm.push(120.0);
  }

  // Per-channel program and note tallies from the merged event stream.
  let mut programs = [0u8; 16];
  let mut notes = [0usize; 16];
  let mut seen = [false; 16];
  for te in &seq.events {
    match te.event {
      Event::NoteOn {
        channel, velocity, ..
      } if velocity > 0 => {
        notes[channel as usize] += 1;
        seen[channel as usize] = true;
      }
      Event::Program { channel, program } => {
        programs[channel as usize] = program;
        seen[channel as usize] = true;
      }
      Event::NoteOn { channel, .. }
      | Event::NoteOff { channel, .. }
      | Event::Cc { channel, .. }
      | Event::PitchBend { channel, .. } => {
        seen[channel as usize] = true;
      }
      Event::EndOfTrack => {}
    }
  }

  let channels: Vec<ChannelInfo> = (0..16u8)
    .filter(|&c| seen[c as usize])
    .map(|c| ChannelInfo {
      channel: c,
      is_drums: c == 9,
      program: programs[c as usize],
      note_count: notes[c as usize],
    })
    .collect();

  Ok(FileInfo {
    format,
    track_count: smf.tracks.len(),
    ppq,
    duration: seq.duration,
    tempos_bpm,
    note_count: notes.iter().sum(),
    channels,
  })
}

pub fn parse(bytes: &[u8]) -> Result<Sequence> {
  let smf = Smf::parse(bytes).map_err(|e| anyhow!("parse MIDI: {e}"))?;

  // Build a tempo map: list of (tick, microseconds-per-quarter) pairs sorted by tick.
  // Format 0/1: tempo events share a single tick timeline across all tracks.
  let mut tempo_changes: Vec<(u32, u32)> = Vec::new();
  for track in smf.tracks.iter() {
    let mut tick: u32 = 0;
    for ev in track.iter() {
      tick = tick.saturating_add(ev.delta.as_int());
      if let TrackEventKind::Meta(MetaMessage::Tempo(uspq)) = ev.kind {
        tempo_changes.push((tick, uspq.as_int()));
      }
    }
  }
  tempo_changes.sort_by_key(|&(t, _)| t);
  // Keep the *last* tempo at each tick (later events override earlier ones).
  tempo_changes.reverse();
  tempo_changes.dedup_by_key(|&mut (t, _)| t);
  tempo_changes.reverse();
  // Default 120 BPM applies only if the file never sets a tempo at tick 0.
  if tempo_changes.first().map(|&(t, _)| t) != Some(0) {
    tempo_changes.insert(0, (0, 500_000));
  }

  let ticks_per_q = match smf.header.timing {
    Timing::Metrical(t) => t.as_int() as u32,
    Timing::Timecode(fps, sub) => {
      // Convert SMPTE to a quasi-PPQ: ticks per second × seconds per quarter.
      // We render through the same code path: pretend 120 BPM → 0.5 s/quarter.
      let tps = (fps.as_f32() * sub as f32) as u32;
      // Tempo of 500_000 µs/q means 0.5 s/q, so ticks_per_q = 0.5 * tps.
      (tps as f32 * 0.5).round() as u32
    }
  }
  .max(1);

  let tick_to_seconds = |tick: u32| -> f64 {
    // Sum durations across tempo segments before `tick`.
    let mut secs = 0.0_f64;
    let mut last_tick = 0u32;
    let mut last_uspq = tempo_changes[0].1;
    for &(t, uspq) in tempo_changes.iter().skip(1) {
      if t >= tick {
        break;
      }
      let span = t - last_tick;
      secs +=
        span as f64 * last_uspq as f64 / (ticks_per_q as f64 * 1_000_000.0);
      last_tick = t;
      last_uspq = uspq;
    }
    let span = tick.saturating_sub(last_tick);
    secs + span as f64 * last_uspq as f64 / (ticks_per_q as f64 * 1_000_000.0)
  };

  // Second pass: merge all tracks' events into absolute-tick timed events.
  let mut events: Vec<(u32, Event)> = Vec::new();
  for track in smf.tracks.iter() {
    let mut tick: u32 = 0;
    for ev in track.iter() {
      tick = tick.saturating_add(ev.delta.as_int());
      match ev.kind {
        TrackEventKind::Midi { channel, message } => {
          let ch = channel.as_int();
          match message {
            MidiMessage::NoteOn { key, vel } => events.push((
              tick,
              Event::NoteOn {
                channel: ch,
                note: key.as_int(),
                velocity: vel.as_int(),
              },
            )),
            MidiMessage::NoteOff { key, vel: _ } => events.push((
              tick,
              Event::NoteOff {
                channel: ch,
                note: key.as_int(),
              },
            )),
            MidiMessage::Controller { controller, value } => events.push((
              tick,
              Event::Cc {
                channel: ch,
                cc: controller.as_int(),
                value: value.as_int(),
              },
            )),
            MidiMessage::ProgramChange { program } => events.push((
              tick,
              Event::Program {
                channel: ch,
                program: program.as_int(),
              },
            )),
            MidiMessage::PitchBend { bend } => {
              let raw = bend.0.as_int() as i32 - 8192;
              let f = raw as f32 / 8192.0;
              events.push((
                tick,
                Event::PitchBend {
                  channel: ch,
                  bend: f,
                },
              ));
            }
            _ => {}
          }
        }
        TrackEventKind::Meta(MetaMessage::EndOfTrack) => {
          events.push((tick, Event::EndOfTrack));
        }
        _ => {}
      }
    }
  }

  // Stable sort by tick — preserves original event order within a tick.
  events.sort_by_key(|(t, _)| *t);

  let mut timed: Vec<TimedEvent> = events
    .into_iter()
    .filter_map(|(tick, e)| match e {
      Event::EndOfTrack => None,
      other => Some(TimedEvent {
        time: tick_to_seconds(tick),
        event: other,
      }),
    })
    .collect();

  let duration = timed.last().map(|e| e.time).unwrap_or(0.0);
  timed.shrink_to_fit();
  Ok(Sequence {
    events: timed,
    duration,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Build a tiny in-memory Format-0 MIDI: 1-quarter rest, then C4 note-on, 1 quarter
  /// later note-off. Default tempo 120 BPM ⇒ 0.5 s/quarter.
  fn tiny_smf() -> Vec<u8> {
    let mut bytes = Vec::new();
    // Header chunk
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&6u32.to_be_bytes());
    bytes.extend_from_slice(&0u16.to_be_bytes()); // format 0
    bytes.extend_from_slice(&1u16.to_be_bytes()); // ntrks
    bytes.extend_from_slice(&480u16.to_be_bytes()); // PPQ
    // Track
    let mut track = Vec::new();
    // delta 0, note-on ch0 C4 vel 100
    track.extend_from_slice(&[0x00, 0x90, 60, 100]);
    // delta 480 (1 quarter), note-off
    track.extend_from_slice(&[0x83, 0x60, 0x80, 60, 0]);
    // delta 0 end-of-track
    track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);
    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(track.len() as u32).to_be_bytes());
    bytes.extend(track);
    bytes
  }

  /// 480 PPQ, Set-Tempo to 1_000_000 µs/quarter (60 BPM) at tick 0,
  /// then note-on at 0 and note-off at 480. One quarter ⇒ 1.0 s.
  fn smf_with_tempo() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&6u32.to_be_bytes());
    bytes.extend_from_slice(&0u16.to_be_bytes()); // format 0
    bytes.extend_from_slice(&1u16.to_be_bytes()); // ntrks
    bytes.extend_from_slice(&480u16.to_be_bytes()); // PPQ
    let mut tr = Vec::new();
    // delta 0, Set-Tempo 1_000_000 µs/q
    tr.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x0F, 0x42, 0x40]);
    // delta 0 note-on
    tr.extend_from_slice(&[0x00, 0x90, 60, 100]);
    // delta 480 note-off
    tr.extend_from_slice(&[0x83, 0x60, 0x80, 60, 0]);
    tr.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);
    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(tr.len() as u32).to_be_bytes());
    bytes.extend(tr);
    bytes
  }

  #[test]
  fn honours_set_tempo_at_tick_zero() {
    let seq = parse(&smf_with_tempo()).expect("parse");
    assert_eq!(seq.events.len(), 2);
    let dt = seq.events[1].time - seq.events[0].time;
    // 60 BPM ⇒ 1 quarter = 1.0 s, not the 0.5 s of the default 120 BPM.
    assert!((dt - 1.0).abs() < 1e-3, "dt = {dt} (expected ≈ 1.0)");
  }

  #[test]
  fn analyze_reports_header_tempo_and_voices() {
    let info = analyze(&smf_with_tempo()).expect("analyze");
    assert_eq!(info.format, 0);
    assert_eq!(info.track_count, 1);
    assert_eq!(info.ppq, Some(480));
    assert_eq!(info.note_count, 1);
    // 60 BPM set at tick 0.
    assert_eq!(info.tempos_bpm.len(), 1);
    assert!((info.tempos_bpm[0] - 60.0).abs() < 1e-3);
    // Only channel 0 is used, and it is not the drum channel.
    assert_eq!(info.channels.len(), 1);
    assert_eq!(info.channels[0].channel, 0);
    assert!(!info.channels[0].is_drums);
    assert_eq!(info.channels[0].note_count, 1);
  }

  #[test]
  fn parses_a_simple_note() {
    let seq = parse(&tiny_smf()).expect("parse");
    assert_eq!(seq.events.len(), 2);
    assert!(matches!(
      seq.events[0].event,
      Event::NoteOn { note: 60, .. }
    ));
    assert!(matches!(
      seq.events[1].event,
      Event::NoteOff { note: 60, .. }
    ));
    // 480 ticks at 120 BPM = 0.5 s
    let dt = seq.events[1].time - seq.events[0].time;
    assert!((dt - 0.5).abs() < 1e-3, "dt = {dt}");
  }
}
