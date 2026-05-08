//! Build a tiny in-memory MIDI file, render it through the full pipeline, and
//! sanity-check the resulting WAV.

use std::path::PathBuf;

fn tiny_smf() -> Vec<u8> {
  // Format 0, 480 PPQ, default 120 BPM (no Set-Tempo meta) → 0.5 s per quarter.
  // Plays a C-major chord (60, 64, 67) for one quarter, then a kick (note 36)
  // on channel 10 (index 9) at the same instant.
  let mut bytes = Vec::new();
  bytes.extend_from_slice(b"MThd");
  bytes.extend_from_slice(&6u32.to_be_bytes());
  bytes.extend_from_slice(&0u16.to_be_bytes()); // format 0
  bytes.extend_from_slice(&1u16.to_be_bytes()); // ntrks
  bytes.extend_from_slice(&480u16.to_be_bytes()); // PPQ

  let mut tr = Vec::new();
  // delta 0, program change ch0 → 0 (acoustic grand)
  tr.extend_from_slice(&[0x00, 0xC0, 0]);
  // delta 0, note-on ch0 60 vel 100
  tr.extend_from_slice(&[0x00, 0x90, 60, 100]);
  // delta 0, note-on ch0 64 vel 100
  tr.extend_from_slice(&[0x00, 0x90, 64, 100]);
  // delta 0, note-on ch0 67 vel 100
  tr.extend_from_slice(&[0x00, 0x90, 67, 100]);
  // delta 0, note-on ch9 36 vel 110 (drum)
  tr.extend_from_slice(&[0x00, 0x99, 36, 110]);
  // delta 480 (1 quarter), note-offs
  tr.extend_from_slice(&[0x83, 0x60, 0x80, 60, 0]); // var-len 480 = 0x83 0x60
  tr.extend_from_slice(&[0x00, 0x80, 64, 0]);
  tr.extend_from_slice(&[0x00, 0x80, 67, 0]);
  tr.extend_from_slice(&[0x00, 0x89, 36, 0]);
  // delta 0 end-of-track
  tr.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

  bytes.extend_from_slice(b"MTrk");
  bytes.extend_from_slice(&(tr.len() as u32).to_be_bytes());
  bytes.extend(tr);
  bytes
}

#[test]
fn renders_non_silent_wav_of_expected_length() {
  let smf_bytes = tiny_smf();
  // Write to a temp dir.
  let dir =
    std::env::temp_dir().join(format!("rumpl-test-{}", std::process::id()));
  std::fs::create_dir_all(&dir).unwrap();
  let mid_path: PathBuf = dir.join("a.mid");
  let wav_path: PathBuf = dir.join("a.wav");
  std::fs::write(&mid_path, &smf_bytes).unwrap();

  // Drive the public render pipeline through `cargo run`'s binary? No — call
  // the library API by re-importing the modules from the binary crate. Since
  // this is a binary crate, we instead invoke the rendered binary.
  let bin = env!("CARGO_BIN_EXE_rumpl");
  let status = std::process::Command::new(bin)
    .args(["render"])
    .arg(&mid_path)
    .arg(&wav_path)
    .arg("--tail")
    .arg("0.5")
    .status()
    .expect("run rumpl render");
  assert!(status.success(), "rumpl render exited with {status}");

  // Check WAV with hound.
  let reader = hound::WavReader::open(&wav_path).expect("open wav");
  let spec = reader.spec();
  assert_eq!(spec.channels, 2);
  assert_eq!(spec.sample_rate, 44_100);
  assert_eq!(spec.bits_per_sample, 16);

  // Expected length: 0.5 s of held notes + 0.5 s tail = 1.0 s = 44_100 frames.
  let frames = reader.duration() as f32;
  let expected = 44_100.0;
  assert!(
    (frames - expected).abs() < 200.0,
    "duration mismatch: got {frames} frames, expected ~{expected}"
  );

  // Peak amplitude > -30 dBFS (i.e. > 1037 in i16).
  let mut reader = hound::WavReader::open(&wav_path).expect("reopen wav");
  let mut peak: i16 = 0;
  for s in reader.samples::<i16>() {
    let s = s.unwrap();
    if s.unsigned_abs() as i32 > peak.unsigned_abs() as i32 {
      peak = s;
    }
  }
  let peak_abs = peak.unsigned_abs() as i32;
  assert!(peak_abs > 1_000, "WAV is silent (peak = {peak_abs})");
  assert!(peak_abs < 32_700, "WAV is clipping at peak = {peak_abs}");

  let _ = std::fs::remove_file(&mid_path);
  let _ = std::fs::remove_file(&wav_path);
}
