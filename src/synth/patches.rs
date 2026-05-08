use crate::synth::env::EnvSpec;
use crate::synth::filter::FilterMode;
use crate::synth::osc::Wave;

#[derive(Clone, Copy, Debug)]
pub struct OscSpec {
  pub wave: Wave,
  pub detune_cents: f32,
  pub mix: f32,
  pub octave: i32,
}

impl OscSpec {
  pub const fn new(wave: Wave, detune_cents: f32, mix: f32) -> Self {
    Self {
      wave,
      detune_cents,
      mix,
      octave: 0,
    }
  }

  pub const fn at(
    wave: Wave,
    detune_cents: f32,
    mix: f32,
    octave: i32,
  ) -> Self {
    Self {
      wave,
      detune_cents,
      mix,
      octave,
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct FilterSpec {
  pub mode: FilterMode,
  pub cutoff: f32,
  pub res: f32,
  pub key_track: f32,  // 0..1
  pub env_amount: f32, // semitones-equivalent multiplier (0..1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LfoTarget {
  Pitch,
  Filter,
  Amp,
}

#[derive(Clone, Copy, Debug)]
pub struct LfoSpec {
  pub rate: f32, // Hz
  pub depth: f32,
  pub target: LfoTarget,
  pub delay: f32, // seconds before LFO ramps in
}

/// Two-operator FM: modulator/carrier ratio and modulation index in radians,
/// both encoded as fixed-point ×10 to keep `const` evaluation simple.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FmSpec {
  pub ratio_x10: u32,
  pub index_x10: u32,
}

impl FmSpec {
  pub const fn new(ratio_x10: u32, index_x10: u32) -> Self {
    Self {
      ratio_x10,
      index_x10,
    }
  }

  pub fn parts(self) -> (f32, f32) {
    (self.ratio_x10 as f32 / 10.0, self.index_x10 as f32 / 10.0)
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Patch {
  pub oscs: &'static [OscSpec],
  pub env: EnvSpec,
  pub filter: Option<FilterSpec>,
  pub lfo: Option<LfoSpec>,
  pub fm: Option<FmSpec>,
  pub gain: f32,
}

const fn p(
  oscs: &'static [OscSpec],
  env: EnvSpec,
  filter: Option<FilterSpec>,
  lfo: Option<LfoSpec>,
  fm: Option<FmSpec>,
  gain: f32,
) -> Patch {
  Patch {
    oscs,
    env,
    filter,
    lfo,
    fm,
    gain,
  }
}

const fn lp(
  cutoff: f32,
  res: f32,
  key_track: f32,
  env_amount: f32,
) -> Option<FilterSpec> {
  Some(FilterSpec {
    mode: FilterMode::Low,
    cutoff,
    res,
    key_track,
    env_amount,
  })
}

const fn vibrato(rate: f32, depth: f32, delay: f32) -> Option<LfoSpec> {
  Some(LfoSpec {
    rate,
    depth,
    target: LfoTarget::Pitch,
    delay,
  })
}

// ---- oscillator stacks ---------------------------------------------------

const SINE: &[OscSpec] = &[OscSpec::new(Wave::Sine, 0.0, 1.0)];
const TRI: &[OscSpec] = &[OscSpec::new(Wave::Triangle, 0.0, 1.0)];
const SAW: &[OscSpec] = &[OscSpec::new(Wave::Saw, 0.0, 1.0)];
const SQR: &[OscSpec] = &[OscSpec::new(Wave::Square, 0.0, 1.0)];

const PIANO_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Triangle, -3.0, 0.7),
  OscSpec::new(Wave::Sine, 3.0, 0.5),
];

const EP_OSC: &[OscSpec] = &[OscSpec::new(Wave::Sine, 0.0, 1.0)];

const ORGAN_OSC: &[OscSpec] = &[
  OscSpec::at(Wave::Sine, 0.0, 0.7, 0), // 8'
  OscSpec::at(Wave::Sine, 0.0, 0.5, 1), // 4'
  OscSpec::at(Wave::Sine, 0.0, 0.3, 2), // 2'
];

const STRING_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, -7.0, 0.6),
  OscSpec::new(Wave::Saw, 7.0, 0.6),
];

const ENS_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, -10.0, 0.5),
  OscSpec::new(Wave::Saw, 0.0, 0.5),
  OscSpec::new(Wave::Saw, 10.0, 0.5),
];

const PAD_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, -12.0, 0.45),
  OscSpec::new(Wave::Saw, 12.0, 0.45),
  OscSpec::at(Wave::Sine, 0.0, 0.3, 1),
];

const SQR_TRI: &[OscSpec] = &[
  OscSpec::new(Wave::Square, 0.0, 0.6),
  OscSpec::new(Wave::Triangle, 0.0, 0.4),
];

const NOISE_OSC: &[OscSpec] = &[OscSpec::new(Wave::Noise, 0.0, 1.0)];

const HONKY_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Triangle, -10.0, 0.6),
  OscSpec::new(Wave::Triangle, 10.0, 0.6),
];

const ACCORDION_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, -5.0, 0.5),
  OscSpec::new(Wave::Saw, 5.0, 0.5),
];

const TANGO_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, -8.0, 0.5),
  OscSpec::new(Wave::Saw, 8.0, 0.5),
];

const CHOIR_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, -5.0, 0.4),
  OscSpec::new(Wave::Saw, 5.0, 0.4),
  OscSpec::new(Wave::Sine, 0.0, 0.3),
];

const FIFTHS_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Saw, 0.0, 0.6),
  OscSpec::at(Wave::Saw, 700.0, 0.5, 0),
];

const CRYSTAL_OSC: &[OscSpec] = &[
  OscSpec::new(Wave::Sine, 0.0, 0.6),
  OscSpec::at(Wave::Sine, 700.0, 0.4, 0),
];

// ---- patch table ---------------------------------------------------------

pub fn gm_patch(program: u8) -> Patch {
  let prog = program.min(127);
  PATCHES[prog as usize]
}

static PATCHES: [Patch; 128] = build_patches();

const fn build_patches() -> [Patch; 128] {
  let mut out = [DEFAULT_PATCH; 128];
  let mut i = 0;
  while i < 128 {
    out[i] = patch_for(i as u8);
    i += 1;
  }
  out
}

const DEFAULT_PATCH: Patch = p(
  TRI,
  EnvSpec::adsr(0.005, 0.4, 0.3, 0.3),
  lp(4_000.0, 0.7, 0.5, 0.0),
  None,
  None,
  0.7,
);

const fn patch_for(prog: u8) -> Patch {
  match prog {
    // 0..7 — Pianos
    0..=2 => p(
      PIANO_OSC,
      EnvSpec::adsr(0.002, 1.2, 0.0, 0.25),
      lp(3_500.0, 0.7, 0.6, 0.4),
      None,
      None,
      0.8,
    ),
    3 => p(
      // honky-tonk: detune more
      HONKY_OSC,
      EnvSpec::adsr(0.002, 1.0, 0.0, 0.2),
      lp(3_500.0, 0.7, 0.6, 0.4),
      None,
      None,
      0.8,
    ),
    4 | 5 => p(
      EP_OSC,
      EnvSpec::adsr(0.002, 1.5, 0.0, 0.3),
      lp(4_000.0, 0.7, 0.4, 0.3),
      None,
      Some(FmSpec::new(10, 30)),
      0.7,
    ),
    6 => p(
      // harpsichord: sharp attack, fast decay, brighter
      SAW,
      EnvSpec::adsr(0.001, 0.5, 0.0, 0.1),
      lp(5_000.0, 0.8, 0.7, 0.0),
      None,
      None,
      0.7,
    ),
    7 => p(
      // clavi
      SAW,
      EnvSpec::adsr(0.002, 0.4, 0.0, 0.1),
      lp(3_000.0, 1.0, 0.7, 0.5),
      None,
      None,
      0.7,
    ),

    // 8..15 — Chromatic percussion
    8 | 9 => p(
      // celesta / glockenspiel
      SINE,
      EnvSpec::adsr(0.001, 1.5, 0.0, 0.5),
      None,
      None,
      Some(FmSpec::new(30, 25)),
      0.7,
    ),
    10 => p(
      // music box
      TRI,
      EnvSpec::adsr(0.001, 0.8, 0.0, 0.3),
      None,
      None,
      Some(FmSpec::new(50, 20)),
      0.7,
    ),
    11 | 12 => p(
      // vibraphone / marimba
      SINE,
      EnvSpec::adsr(0.002, 1.5, 0.0, 0.5),
      None,
      Some(LfoSpec {
        rate: 5.0,
        depth: 0.05,
        target: LfoTarget::Amp,
        delay: 0.1,
      }),
      Some(FmSpec::new(40, 15)),
      0.7,
    ),
    13 | 14 => p(
      // xylophone / tubular bells
      SINE,
      EnvSpec::adsr(0.001, 1.0, 0.0, 0.3),
      None,
      None,
      Some(FmSpec::new(35, 30)),
      0.7,
    ),
    15 => p(
      // dulcimer
      TRI,
      EnvSpec::adsr(0.001, 1.2, 0.0, 0.3),
      lp(4_000.0, 0.7, 0.6, 0.0),
      None,
      None,
      0.7,
    ),

    // 16..23 — Organs
    16..=18 => p(
      ORGAN_OSC,
      EnvSpec::adsr(0.01, 0.05, 0.95, 0.05),
      None,
      None,
      None,
      0.6,
    ),
    19 | 20 => p(
      // church/reed organ
      ORGAN_OSC,
      EnvSpec::adsr(0.05, 0.1, 0.95, 0.15),
      lp(3_500.0, 0.7, 0.5, 0.0),
      None,
      None,
      0.6,
    ),
    21 | 22 => p(
      // accordion / harmonica
      ACCORDION_OSC,
      EnvSpec::adsr(0.05, 0.1, 0.85, 0.1),
      lp(3_000.0, 0.8, 0.5, 0.0),
      vibrato(5.5, 0.02, 0.2),
      None,
      0.6,
    ),
    23 => p(
      // tango accordion
      TANGO_OSC,
      EnvSpec::adsr(0.05, 0.1, 0.85, 0.15),
      lp(2_800.0, 0.9, 0.5, 0.0),
      vibrato(6.0, 0.025, 0.2),
      None,
      0.6,
    ),

    // 24..31 — Guitars
    24 | 25 => p(
      // nylon / steel
      TRI,
      EnvSpec::adsr(0.005, 1.0, 0.0, 0.2),
      lp(2_500.0, 0.7, 0.6, 0.5),
      None,
      None,
      0.7,
    ),
    26 | 27 => p(
      // jazz / clean
      SAW,
      EnvSpec::adsr(0.005, 0.8, 0.0, 0.2),
      lp(2_500.0, 0.8, 0.6, 0.5),
      None,
      None,
      0.6,
    ),
    28 | 29 => p(
      // muted / overdriven
      SAW,
      EnvSpec::adsr(0.005, 0.4, 0.2, 0.1),
      lp(1_800.0, 1.5, 0.5, 0.5),
      None,
      None,
      0.6,
    ),
    30 | 31 => p(
      // distortion / harmonics
      SAW,
      EnvSpec::adsr(0.01, 1.5, 0.7, 0.2),
      lp(2_500.0, 2.0, 0.5, 0.4),
      None,
      None,
      0.5,
    ),

    // 32..39 — Bass
    32 | 33 => p(
      // acoustic / fingered
      TRI,
      EnvSpec::adsr(0.005, 0.6, 0.3, 0.1),
      lp(1_500.0, 0.9, 0.5, 0.6),
      None,
      None,
      0.8,
    ),
    34 | 35 => p(
      // picked / fretless
      SAW,
      EnvSpec::adsr(0.005, 0.5, 0.3, 0.1),
      lp(1_400.0, 1.0, 0.5, 0.6),
      None,
      None,
      0.7,
    ),
    36 | 37 => p(
      // slap
      SAW,
      EnvSpec::adsr(0.002, 0.3, 0.1, 0.05),
      lp(2_000.0, 1.5, 0.6, 0.7),
      None,
      None,
      0.7,
    ),
    38 | 39 => p(
      // synth bass 1/2
      SQR,
      EnvSpec::adsr(0.005, 0.4, 0.4, 0.1),
      lp(1_200.0, 1.5, 0.4, 0.6),
      None,
      None,
      0.7,
    ),

    // 40..47 — Solo strings
    40 | 41 => p(
      // violin / viola
      STRING_OSC,
      EnvSpec::adsr(0.08, 0.3, 0.85, 0.2),
      lp(3_500.0, 0.7, 0.5, 0.0),
      vibrato(5.5, 0.02, 0.4),
      None,
      0.6,
    ),
    42 | 43 => p(
      // cello / contrabass
      STRING_OSC,
      EnvSpec::adsr(0.06, 0.3, 0.85, 0.2),
      lp(2_500.0, 0.7, 0.5, 0.0),
      vibrato(5.0, 0.02, 0.3),
      None,
      0.6,
    ),
    44 => p(
      // tremolo strings
      STRING_OSC,
      EnvSpec::adsr(0.05, 0.2, 0.85, 0.15),
      lp(3_000.0, 0.7, 0.5, 0.0),
      Some(LfoSpec {
        rate: 7.0,
        depth: 0.4,
        target: LfoTarget::Amp,
        delay: 0.0,
      }),
      None,
      0.6,
    ),
    45 => p(
      // pizzicato strings
      TRI,
      EnvSpec::adsr(0.005, 0.3, 0.0, 0.1),
      lp(2_500.0, 0.7, 0.6, 0.4),
      None,
      None,
      0.7,
    ),
    46 => p(
      // orchestral harp
      TRI,
      EnvSpec::adsr(0.002, 1.5, 0.0, 0.3),
      lp(3_000.0, 0.7, 0.6, 0.0),
      None,
      None,
      0.7,
    ),
    47 => p(
      // timpani
      SINE,
      EnvSpec::adsr(0.005, 0.8, 0.0, 0.3),
      None,
      None,
      Some(FmSpec::new(14, 50)),
      0.8,
    ),

    // 48..55 — Ensemble
    48 | 49 => p(
      // string ensembles
      ENS_OSC,
      EnvSpec::adsr(0.15, 0.4, 0.85, 0.4),
      lp(3_500.0, 0.7, 0.5, 0.0),
      vibrato(5.0, 0.015, 0.3),
      None,
      0.55,
    ),
    50 | 51 => p(
      // synth strings
      ENS_OSC,
      EnvSpec::adsr(0.2, 0.5, 0.85, 0.5),
      lp(3_000.0, 0.8, 0.5, 0.0),
      vibrato(5.5, 0.02, 0.3),
      None,
      0.55,
    ),
    52 | 53 => p(
      // choir aahs / voice oohs
      CHOIR_OSC,
      EnvSpec::adsr(0.12, 0.3, 0.85, 0.4),
      lp(2_200.0, 1.2, 0.5, 0.0),
      vibrato(5.5, 0.015, 0.4),
      None,
      0.55,
    ),
    54 => p(
      // synth voice
      ENS_OSC,
      EnvSpec::adsr(0.1, 0.3, 0.85, 0.3),
      lp(2_500.0, 1.0, 0.5, 0.0),
      vibrato(5.5, 0.02, 0.3),
      None,
      0.55,
    ),
    55 => p(
      // orchestra hit
      ENS_OSC,
      EnvSpec::adsr(0.005, 0.6, 0.0, 0.2),
      lp(3_500.0, 0.8, 0.5, 0.5),
      None,
      None,
      0.7,
    ),

    // 56..63 — Brass
    56..=59 => p(
      // trumpet / trombone / tuba / muted trumpet
      SAW,
      EnvSpec::adsr(0.04, 0.15, 0.9, 0.15),
      lp(2_500.0, 0.9, 0.7, 0.5),
      vibrato(5.0, 0.012, 0.35),
      None,
      0.6,
    ),
    60 | 61 => p(
      // french horn / brass section
      SAW,
      EnvSpec::adsr(0.06, 0.2, 0.9, 0.2),
      lp(2_200.0, 0.8, 0.6, 0.4),
      vibrato(5.0, 0.01, 0.3),
      None,
      0.6,
    ),
    62 | 63 => p(
      // synth brass
      SAW,
      EnvSpec::adsr(0.04, 0.2, 0.85, 0.15),
      lp(2_500.0, 1.2, 0.6, 0.6),
      None,
      None,
      0.6,
    ),

    // 64..71 — Reed
    64..=67 => p(
      // saxophones
      SQR_TRI,
      EnvSpec::adsr(0.05, 0.15, 0.85, 0.15),
      lp(2_800.0, 1.0, 0.6, 0.4),
      vibrato(5.0, 0.012, 0.3),
      None,
      0.6,
    ),
    68 | 69 => p(
      // oboe / english horn
      SQR,
      EnvSpec::adsr(0.04, 0.15, 0.85, 0.15),
      lp(3_000.0, 0.9, 0.6, 0.3),
      vibrato(5.5, 0.012, 0.3),
      None,
      0.55,
    ),
    70 | 71 => p(
      // bassoon / clarinet
      SQR,
      EnvSpec::adsr(0.05, 0.15, 0.85, 0.15),
      lp(2_500.0, 0.8, 0.6, 0.3),
      vibrato(5.0, 0.01, 0.3),
      None,
      0.55,
    ),

    // 72..79 — Pipe
    72..=75 => p(
      // piccolo / flute / recorder / pan flute
      TRI,
      EnvSpec::adsr(0.06, 0.1, 0.9, 0.15),
      lp(4_000.0, 0.7, 0.6, 0.0),
      vibrato(5.5, 0.02, 0.3),
      None,
      0.55,
    ),
    76..=79 => p(
      // bottle / shakuhachi / whistle / ocarina
      SINE,
      EnvSpec::adsr(0.08, 0.1, 0.85, 0.2),
      lp(3_500.0, 0.7, 0.5, 0.0),
      vibrato(5.5, 0.02, 0.3),
      None,
      0.55,
    ),

    // 80..87 — Synth lead
    80 => p(
      // square lead
      SQR,
      EnvSpec::adsr(0.005, 0.1, 0.9, 0.1),
      lp(3_000.0, 0.9, 0.5, 0.0),
      None,
      None,
      0.6,
    ),
    81 => p(
      // saw lead
      SAW,
      EnvSpec::adsr(0.005, 0.1, 0.9, 0.1),
      lp(3_500.0, 0.9, 0.5, 0.0),
      None,
      None,
      0.6,
    ),
    82 | 83 => p(
      // calliope / chiff
      TRI,
      EnvSpec::adsr(0.01, 0.1, 0.9, 0.15),
      lp(2_500.0, 0.8, 0.5, 0.0),
      vibrato(5.5, 0.015, 0.2),
      None,
      0.55,
    ),
    84 | 85 => p(
      // charang / voice
      SAW,
      EnvSpec::adsr(0.01, 0.2, 0.85, 0.15),
      lp(2_500.0, 1.2, 0.5, 0.0),
      vibrato(5.0, 0.015, 0.3),
      None,
      0.55,
    ),
    86 | 87 => p(
      // fifths / bass+lead
      FIFTHS_OSC,
      EnvSpec::adsr(0.01, 0.2, 0.85, 0.15),
      lp(2_500.0, 1.0, 0.5, 0.0),
      None,
      None,
      0.55,
    ),

    // 88..95 — Synth pad
    88..=95 => p(
      PAD_OSC,
      EnvSpec::adsr(0.6, 1.0, 0.8, 1.0),
      lp(2_000.0, 1.2, 0.5, 0.4),
      Some(LfoSpec {
        rate: 0.5,
        depth: 200.0, // cutoff modulation in Hz
        target: LfoTarget::Filter,
        delay: 0.0,
      }),
      None,
      0.5,
    ),

    // 96..103 — Synth FX
    96 | 97 => p(
      // rain / soundtrack
      PAD_OSC,
      EnvSpec::adsr(1.0, 1.5, 0.7, 1.5),
      lp(1_800.0, 1.2, 0.5, 0.5),
      Some(LfoSpec {
        rate: 0.3,
        depth: 300.0,
        target: LfoTarget::Filter,
        delay: 0.0,
      }),
      None,
      0.5,
    ),
    98 | 99 => p(
      // crystal / atmosphere
      CRYSTAL_OSC,
      EnvSpec::adsr(0.05, 1.5, 0.4, 1.0),
      lp(3_000.0, 1.0, 0.5, 0.0),
      None,
      Some(FmSpec::new(30, 30)),
      0.55,
    ),
    100 | 101 => p(
      // brightness / goblins
      SAW,
      EnvSpec::adsr(0.5, 1.0, 0.8, 1.0),
      lp(2_500.0, 1.5, 0.5, 0.5),
      Some(LfoSpec {
        rate: 1.0,
        depth: 400.0,
        target: LfoTarget::Filter,
        delay: 0.0,
      }),
      None,
      0.5,
    ),
    102 | 103 => p(
      // echoes / sci-fi
      PAD_OSC,
      EnvSpec::adsr(0.4, 1.0, 0.6, 1.5),
      lp(2_500.0, 1.0, 0.5, 0.3),
      Some(LfoSpec {
        rate: 0.4,
        depth: 300.0,
        target: LfoTarget::Filter,
        delay: 0.0,
      }),
      None,
      0.5,
    ),

    // 104..111 — Ethnic
    104..=109 => p(
      // sitar / banjo / shamisen / koto / kalimba / bagpipe
      SAW,
      EnvSpec::adsr(0.005, 0.6, 0.0, 0.2),
      lp(2_500.0, 0.9, 0.6, 0.5),
      None,
      None,
      0.65,
    ),
    110 | 111 => p(
      // fiddle / shanai
      SAW,
      EnvSpec::adsr(0.05, 0.2, 0.85, 0.2),
      lp(2_800.0, 0.9, 0.6, 0.3),
      vibrato(5.5, 0.018, 0.3),
      None,
      0.6,
    ),

    // 112..119 — Percussive
    112 => p(
      // tinkle bell
      SINE,
      EnvSpec::adsr(0.001, 1.5, 0.0, 0.5),
      None,
      None,
      Some(FmSpec::new(35, 30)),
      0.7,
    ),
    113 | 114 => p(
      // agogo / steel drums
      TRI,
      EnvSpec::adsr(0.002, 0.6, 0.0, 0.2),
      lp(3_000.0, 0.9, 0.6, 0.0),
      None,
      Some(FmSpec::new(25, 20)),
      0.7,
    ),
    115 | 116 => p(
      // woodblock / taiko
      TRI,
      EnvSpec::adsr(0.002, 0.3, 0.0, 0.1),
      lp(2_000.0, 1.5, 0.5, 0.0),
      None,
      None,
      0.7,
    ),
    117 | 118 => p(
      // melodic / synth toms
      SINE,
      EnvSpec::adsr(0.002, 0.4, 0.0, 0.1),
      lp(1_500.0, 1.5, 0.5, 0.0),
      None,
      None,
      0.7,
    ),
    119 => p(
      // reverse cymbal — very rough approximation
      NOISE_OSC,
      EnvSpec::adsr(0.8, 0.05, 0.0, 0.05),
      Some(FilterSpec {
        mode: FilterMode::High,
        cutoff: 5_000.0,
        res: 0.7,
        key_track: 0.0,
        env_amount: 0.0,
      }),
      None,
      None,
      0.4,
    ),

    // 120..127 — Sound FX
    120 => p(
      // guitar fret noise
      NOISE_OSC,
      EnvSpec::adsr(0.001, 0.15, 0.0, 0.05),
      lp(2_500.0, 0.7, 0.0, 0.0),
      None,
      None,
      0.4,
    ),
    121 => p(
      // breath noise
      NOISE_OSC,
      EnvSpec::adsr(0.05, 0.5, 0.0, 0.1),
      Some(FilterSpec {
        mode: FilterMode::Band,
        cutoff: 1_500.0,
        res: 1.0,
        key_track: 0.0,
        env_amount: 0.0,
      }),
      None,
      None,
      0.35,
    ),
    122 => p(
      // seashore
      NOISE_OSC,
      EnvSpec::adsr(0.5, 1.0, 0.6, 1.0),
      lp(1_500.0, 0.7, 0.0, 0.0),
      Some(LfoSpec {
        rate: 0.2,
        depth: 500.0,
        target: LfoTarget::Filter,
        delay: 0.0,
      }),
      None,
      0.4,
    ),
    123 => p(
      // bird tweet — high FM
      SINE,
      EnvSpec::adsr(0.005, 0.2, 0.0, 0.05),
      None,
      Some(LfoSpec {
        rate: 12.0,
        depth: 0.05,
        target: LfoTarget::Pitch,
        delay: 0.0,
      }),
      Some(FmSpec::new(30, 30)),
      0.5,
    ),
    124 => p(
      // telephone ring
      SINE,
      EnvSpec::adsr(0.01, 0.05, 0.85, 0.05),
      None,
      None,
      None,
      0.5,
    ),
    125 => p(
      // helicopter
      NOISE_OSC,
      EnvSpec::adsr(0.05, 0.1, 0.85, 0.2),
      lp(800.0, 1.5, 0.0, 0.0),
      Some(LfoSpec {
        rate: 8.0,
        depth: 0.7,
        target: LfoTarget::Amp,
        delay: 0.0,
      }),
      None,
      0.5,
    ),
    126 => p(
      // applause
      NOISE_OSC,
      EnvSpec::adsr(0.3, 0.5, 0.6, 0.5),
      Some(FilterSpec {
        mode: FilterMode::High,
        cutoff: 1_500.0,
        res: 0.7,
        key_track: 0.0,
        env_amount: 0.0,
      }),
      None,
      None,
      0.4,
    ),
    _ => p(
      // 127 — gunshot
      NOISE_OSC,
      EnvSpec::adsr(0.001, 0.2, 0.0, 0.05),
      lp(2_000.0, 1.5, 0.0, 0.5),
      None,
      None,
      0.6,
    ),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_program_has_a_patch() {
    for prog in 0u8..128 {
      let pa = gm_patch(prog);
      assert!(!pa.oscs.is_empty(), "program {prog} has no oscillators");
      assert!(pa.gain > 0.0, "program {prog} has zero gain");
    }
  }
}
