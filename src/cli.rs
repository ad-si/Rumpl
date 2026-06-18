use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
  name = "rumpl",
  version,
  about = "Sample-free General MIDI player and renderer"
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
  /// Play a MIDI file in real time.
  Play {
    /// Input .mid file
    file: PathBuf,
    /// Master gain (0..1)
    #[arg(long, default_value_t = 0.8)]
    gain: f32,
    /// Seconds of tail rendered after the last event
    #[arg(long, default_value_t = 3.0)]
    tail: f32,
    /// Accepted but ignored — rumpl does not use SoundFonts.
    #[arg(long, hide = true)]
    soundfont: Option<PathBuf>,
  },
  /// Show information about a MIDI file (length, tempo, voices).
  Info {
    /// Input .mid file
    file: PathBuf,
  },
  /// List all available General MIDI instruments.
  Instruments,
  /// Render a MIDI file to a 16-bit stereo WAV.
  Render {
    /// Input .mid file
    file: PathBuf,
    /// Output .wav file
    out: PathBuf,
    /// Master gain (0..1)
    #[arg(long, default_value_t = 0.8)]
    gain: f32,
    /// Output sample rate (Hz)
    #[arg(long, default_value_t = 44_100)]
    sample_rate: u32,
    /// Seconds of tail rendered after the last event
    #[arg(long, default_value_t = 3.0)]
    tail: f32,
    /// Accepted but ignored — rumpl does not use SoundFonts.
    #[arg(long, hide = true)]
    soundfont: Option<PathBuf>,
  },
}
