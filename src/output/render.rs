use anyhow::{Context, Result};
use hound::{SampleFormat, WavSpec, WavWriter};

use crate::midi::Sequence;
use crate::sequencer::Sequencer;
use crate::synth::Synth;

const BLOCK_FRAMES: usize = 512;

pub fn render_to_wav(
  seq: &Sequence,
  out_path: &std::path::Path,
  sample_rate: u32,
  gain: f32,
  tail_seconds: f32,
) -> Result<()> {
  let spec = WavSpec {
    channels: 2,
    sample_rate,
    bits_per_sample: 16,
    sample_format: SampleFormat::Int,
  };
  let mut writer = WavWriter::create(out_path, spec)
    .with_context(|| format!("opening {}", out_path.display()))?;

  let mut synth = Synth::new(sample_rate as f32);
  synth.master_gain = gain;
  let mut seq_iter = Sequencer::new(seq, sample_rate, tail_seconds);

  let mut buf = vec![0.0f32; BLOCK_FRAMES * 2];
  let total = seq_iter.total_samples();
  let mut written: u64 = 0;
  while written < total {
    let frames = ((total - written) as usize).min(BLOCK_FRAMES);
    let slice = &mut buf[..frames * 2];
    seq_iter.render_block(&mut synth, slice);
    for &s in slice.iter() {
      let clipped = s.clamp(-1.0, 1.0);
      let i = (clipped * 32_767.0).round() as i16;
      writer.write_sample(i)?;
    }
    written += frames as u64;
  }

  writer.finalize().context("finalizing WAV")?;
  Ok(())
}
