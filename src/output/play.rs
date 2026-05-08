use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Observer, Producer, Split};

use crate::midi::Sequence;
use crate::sequencer::Sequencer;
use crate::synth::Synth;

const RING_FRAMES: usize = 8_192; // ~185 ms at 44.1 kHz
const BLOCK_FRAMES: usize = 256;

pub fn play(seq: Sequence, gain: f32, tail_seconds: f32) -> Result<()> {
  let host = cpal::default_host();
  let device = host
    .default_output_device()
    .ok_or_else(|| anyhow!("no default output device"))?;
  let supported = device
    .default_output_config()
    .context("default output config")?;
  let sample_format = supported.sample_format();
  let mut config: StreamConfig = supported.into();
  // Force stereo where supported.
  if config.channels < 2 {
    return Err(anyhow!("audio device has < 2 channels"));
  }
  let channels = config.channels as usize;
  let sr = config.sample_rate.0;
  config.buffer_size = cpal::BufferSize::Default;

  let rb = HeapRb::<f32>::new(RING_FRAMES * 2);
  let (mut prod, mut cons) = rb.split();

  let stop = Arc::new(AtomicBool::new(false));
  let stop_audio = stop.clone();
  let stop_thread = stop.clone();

  let err_fn = |e| eprintln!("audio stream error: {e}");

  let stream = match sample_format {
    SampleFormat::F32 => device.build_output_stream(
      &config,
      move |data: &mut [f32], _| {
        fill(data, channels, &mut cons, &stop_audio);
      },
      err_fn,
      None,
    ),
    SampleFormat::I16 => device.build_output_stream(
      &config,
      move |data: &mut [i16], _| {
        let mut tmp = vec![0.0f32; data.len()];
        fill(&mut tmp, channels, &mut cons, &stop_audio);
        for (o, &f) in data.iter_mut().zip(tmp.iter()) {
          *o = (f.clamp(-1.0, 1.0) * 32_767.0) as i16;
        }
      },
      err_fn,
      None,
    ),
    SampleFormat::U16 => device.build_output_stream(
      &config,
      move |data: &mut [u16], _| {
        let mut tmp = vec![0.0f32; data.len()];
        fill(&mut tmp, channels, &mut cons, &stop_audio);
        for (o, &f) in data.iter_mut().zip(tmp.iter()) {
          *o = ((f.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16;
        }
      },
      err_fn,
      None,
    ),
    other => return Err(anyhow!("unsupported sample format: {other:?}")),
  }
  .context("building output stream")?;

  stream.play().context("starting output stream")?;

  // Sequencer thread fills the ring buffer.
  let producer_thread = thread::spawn(move || {
    let mut synth = Synth::new(sr as f32);
    synth.master_gain = gain;
    let mut seqr = Sequencer::new(&seq, sr, tail_seconds);
    let mut buf = vec![0.0f32; BLOCK_FRAMES * 2];
    let total = seqr.total_samples();
    let mut written: u64 = 0;
    while written < total && !stop_thread.load(Ordering::Relaxed) {
      // Wait until the ring has room for a full block.
      while prod.vacant_len() < buf.len()
        && !stop_thread.load(Ordering::Relaxed)
      {
        thread::sleep(Duration::from_millis(2));
      }
      if stop_thread.load(Ordering::Relaxed) {
        break;
      }
      let frames = ((total - written) as usize).min(BLOCK_FRAMES);
      let slice = &mut buf[..frames * 2];
      seqr.render_block(&mut synth, slice);
      let mut pushed = 0;
      while pushed < slice.len() {
        pushed += prod.push_slice(&slice[pushed..]);
        if stop_thread.load(Ordering::Relaxed) {
          break;
        }
        if pushed < slice.len() {
          thread::sleep(Duration::from_millis(2));
        }
      }
      written += frames as u64;
    }
    // Pad with silence so the consumer doesn't underrun before the song ends.
  });

  // Wait for the producer to finish, plus a small drain margin.
  producer_thread
    .join()
    .map_err(|_| anyhow!("producer thread panicked"))?;
  thread::sleep(Duration::from_millis(200));
  stop.store(true, Ordering::Relaxed);
  drop(stream);
  Ok(())
}

fn fill(
  out: &mut [f32],
  channels: usize,
  cons: &mut ringbuf::HeapCons<f32>,
  stop: &AtomicBool,
) {
  if channels == 2 {
    let popped = cons.pop_slice(out);
    for s in &mut out[popped..] {
      *s = 0.0;
    }
    if stop.load(Ordering::Relaxed) && popped == 0 {
      for s in out.iter_mut() {
        *s = 0.0;
      }
    }
    return;
  }
  // For non-stereo devices, pull stereo pairs and downmix / fan-out.
  let frames = out.len() / channels;
  let mut stereo = vec![0.0f32; frames * 2];
  let popped = cons.pop_slice(&mut stereo);
  for s in &mut stereo[popped..] {
    *s = 0.0;
  }
  for f in 0..frames {
    let l = stereo[f * 2];
    let r = stereo[f * 2 + 1];
    let mono = (l + r) * 0.5;
    for c in 0..channels {
      out[f * channels + c] = if channels == 1 {
        mono
      } else if c == 0 {
        l
      } else if c == 1 {
        r
      } else {
        mono
      };
    }
  }
}
