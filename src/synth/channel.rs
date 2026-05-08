#[derive(Clone, Debug)]
pub struct Channel {
  pub program: u8,
  pub volume: f32,     // CC7 0..1
  pub expression: f32, // CC11 0..1
  pub pan: f32,        // -1 (L) .. 1 (R)
  pub mod_wheel: f32,  // CC1 0..1
  pub sustain: bool,   // CC64
  pub pitch_bend: f32, // -1..1 (semitones range applied separately)
  pub bend_range: f32, // semitones, default 2
  /// True for the GM percussion channel (channel 10, index 9).
  pub is_drums: bool,
  /// RPN selector state (LSB, MSB) for parsing pitch-bend range etc.
  pub rpn_msb: u8,
  pub rpn_lsb: u8,
}

impl Channel {
  pub fn new(index: u8) -> Self {
    Self {
      program: 0,
      volume: 100.0 / 127.0,
      expression: 1.0,
      pan: 0.0,
      mod_wheel: 0.0,
      sustain: false,
      pitch_bend: 0.0,
      bend_range: 2.0,
      is_drums: index == 9,
      rpn_msb: 0x7F,
      rpn_lsb: 0x7F,
    }
  }

  /// Combined linear gain from CC7 × CC11.
  pub fn gain(&self) -> f32 {
    self.volume * self.expression
  }

  /// Pitch-bend in semitones.
  pub fn bend_semitones(&self) -> f32 {
    self.pitch_bend * self.bend_range
  }
}
