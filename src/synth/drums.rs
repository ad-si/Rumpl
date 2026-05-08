use crate::synth::env::EnvSpec;
use crate::synth::filter::FilterMode;
use crate::synth::osc::Wave;

#[derive(Clone, Copy, Debug)]
pub struct DrumTone {
  pub wave: Wave,
  pub freq: f32,
  pub mix: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct DrumFilter {
  pub mode: FilterMode,
  pub cutoff: f32,
  pub res: f32,
}

/// Initial pitch multiplier; sweeps to 1.0 over `time` seconds.
#[derive(Clone, Copy, Debug)]
pub struct DrumSweep {
  pub from: f32,
  pub time: f32,
}

/// A simple percussion voice: a tonal layer (optional) + a noise layer (optional)
/// + a downward pitch sweep + amplitude envelope. Filter is always present.
#[derive(Clone, Copy, Debug)]
pub struct DrumPatch {
  pub tone: Option<DrumTone>,
  pub noise_mix: f32,
  pub filter: DrumFilter,
  pub env: EnvSpec,
  pub sweep: DrumSweep,
  pub gain: f32,
  /// Mute group: voices with the same non-zero group choke each other.
  pub group: u8,
}

const fn drum(
  tone: Option<DrumTone>,
  noise_mix: f32,
  filter: DrumFilter,
  env: EnvSpec,
  sweep: DrumSweep,
  gain: f32,
) -> DrumPatch {
  DrumPatch {
    tone,
    noise_mix,
    filter,
    env,
    sweep,
    gain,
    group: 0,
  }
}

const fn group(mut p: DrumPatch, g: u8) -> DrumPatch {
  p.group = g;
  p
}

pub fn drum_patch(note: u8) -> DrumPatch {
  use FilterMode::*;
  use Wave::*;
  match note {
    // 27 high q
    27 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 1_800.0,
        mix: 0.6,
      }),
      0.4,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 4.0,
      },
      EnvSpec::ad(0.001, 0.05),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 28 slap
    28 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 4_000.0,
        res: 1.0,
      },
      EnvSpec::ad(0.001, 0.04),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 29/30 scratch push/pull
    29 | 30 => drum(
      None,
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.005, 0.08),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 31 sticks
    31 => drum(
      Some(DrumTone {
        wave: Triangle,
        freq: 1_200.0,
        mix: 0.4,
      }),
      0.6,
      DrumFilter {
        mode: Band,
        cutoff: 4_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.001, 0.03),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 32 square click
    32 => drum(
      Some(DrumTone {
        wave: Square,
        freq: 1_500.0,
        mix: 0.6,
      }),
      0.4,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 2.0,
      },
      EnvSpec::ad(0.001, 0.03),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 33 metronome click
    33 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 2_000.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 4.0,
      },
      EnvSpec::ad(0.001, 0.04),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 34 metronome bell
    34 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 2_500.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 6.0,
      },
      EnvSpec::ad(0.001, 0.4),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),

    // 35 acoustic bass drum, 36 bass drum 1
    35 | 36 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 55.0,
        mix: 0.9,
      }),
      0.2,
      DrumFilter {
        mode: Low,
        cutoff: 1_800.0,
        res: 0.7,
      },
      EnvSpec::ad(0.001, 0.25),
      DrumSweep {
        from: 4.0,
        time: 0.04,
      },
      1.0,
    ),

    // 37 side stick
    37 => drum(
      Some(DrumTone {
        wave: Triangle,
        freq: 800.0,
        mix: 0.5,
      }),
      0.5,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.001, 0.04),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),

    // 38 acoustic snare
    38 => drum(
      Some(DrumTone {
        wave: Triangle,
        freq: 200.0,
        mix: 0.6,
      }),
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 2_000.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.18),
      DrumSweep {
        from: 2.0,
        time: 0.02,
      },
      0.85,
    ),
    // 39 hand clap
    39 => drum(
      None,
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 1_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.002, 0.12),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.7,
    ),
    // 40 electric snare
    40 => drum(
      Some(DrumTone {
        wave: Square,
        freq: 250.0,
        mix: 0.5,
      }),
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.15),
      DrumSweep {
        from: 2.0,
        time: 0.02,
      },
      0.8,
    ),

    // 41 low floor tom
    41 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 80.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 1_500.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.5),
      DrumSweep {
        from: 2.5,
        time: 0.05,
      },
      0.85,
    ),
    // 42 closed hi-hat
    42 => group(
      drum(
        None,
        1.0,
        DrumFilter {
          mode: High,
          cutoff: 8_000.0,
          res: 1.0,
        },
        EnvSpec::ad(0.001, 0.05),
        DrumSweep {
          from: 1.0,
          time: 0.01,
        },
        0.55,
      ),
      1,
    ),
    // 43 high floor tom
    43 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 100.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 1_500.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.45),
      DrumSweep {
        from: 2.5,
        time: 0.05,
      },
      0.85,
    ),
    // 44 pedal hi-hat
    44 => group(
      drum(
        None,
        1.0,
        DrumFilter {
          mode: High,
          cutoff: 7_500.0,
          res: 1.0,
        },
        EnvSpec::ad(0.001, 0.08),
        DrumSweep {
          from: 1.0,
          time: 0.01,
        },
        0.5,
      ),
      1,
    ),
    // 45 low tom
    45 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 120.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 1_800.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.4),
      DrumSweep {
        from: 2.5,
        time: 0.05,
      },
      0.85,
    ),
    // 46 open hi-hat
    46 => group(
      drum(
        None,
        1.0,
        DrumFilter {
          mode: High,
          cutoff: 7_000.0,
          res: 1.0,
        },
        EnvSpec::ad(0.001, 0.4),
        DrumSweep {
          from: 1.0,
          time: 0.01,
        },
        0.55,
      ),
      1,
    ),
    // 47 low-mid tom
    47 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 150.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 1_800.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.35),
      DrumSweep {
        from: 2.5,
        time: 0.05,
      },
      0.85,
    ),
    // 48 hi-mid tom
    48 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 180.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 2_000.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.3),
      DrumSweep {
        from: 2.5,
        time: 0.05,
      },
      0.85,
    ),
    // 49 crash 1
    49 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 5_000.0,
        res: 0.7,
      },
      EnvSpec::ad(0.002, 1.5),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.55,
    ),
    // 50 high tom
    50 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 220.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 2_200.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.28),
      DrumSweep {
        from: 2.5,
        time: 0.05,
      },
      0.8,
    ),
    // 51 ride 1
    51 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 5_000.0,
        mix: 0.4,
      }),
      0.6,
      DrumFilter {
        mode: High,
        cutoff: 4_500.0,
        res: 0.7,
      },
      EnvSpec::ad(0.002, 1.2),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 52 chinese cymbal
    52 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 4_000.0,
        res: 0.7,
      },
      EnvSpec::ad(0.002, 1.4),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 53 ride bell
    53 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 4_000.0,
        mix: 0.6,
      }),
      0.4,
      DrumFilter {
        mode: High,
        cutoff: 4_500.0,
        res: 1.0,
      },
      EnvSpec::ad(0.002, 0.6),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.55,
    ),
    // 54 tambourine
    54 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 6_000.0,
        res: 1.0,
      },
      EnvSpec::ad(0.002, 0.15),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.55,
    ),
    // 55 splash cymbal
    55 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 6_000.0,
        res: 0.7,
      },
      EnvSpec::ad(0.002, 0.6),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 56 cowbell
    56 => drum(
      Some(DrumTone {
        wave: Square,
        freq: 800.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 2.5,
      },
      EnvSpec::ad(0.001, 0.2),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 57 crash 2
    57 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 5_500.0,
        res: 0.7,
      },
      EnvSpec::ad(0.002, 1.4),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.55,
    ),
    // 58 vibraslap
    58 => drum(
      None,
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 1.5,
      },
      EnvSpec::ad(0.005, 0.5),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 59 ride 2
    59 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 4_500.0,
        mix: 0.4,
      }),
      0.6,
      DrumFilter {
        mode: High,
        cutoff: 4_500.0,
        res: 0.7,
      },
      EnvSpec::ad(0.002, 1.0),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 60 high bongo
    60 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 350.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_500.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.2),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 61 low bongo
    61 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 280.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_000.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.22),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 62 mute hi conga
    62 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 250.0,
        mix: 0.8,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.18),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 63 open hi conga
    63 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 220.0,
        mix: 0.8,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.3),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 64 low conga
    64 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 170.0,
        mix: 0.8,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_000.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.32),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 65 high timbale
    65 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 300.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 3_000.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.3),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 66 low timbale
    66 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 240.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 1.5,
      },
      EnvSpec::ad(0.001, 0.32),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.7,
    ),
    // 67 high agogo
    67 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 1_200.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_500.0,
        res: 3.0,
      },
      EnvSpec::ad(0.001, 0.25),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 68 low agogo
    68 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 900.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_000.0,
        res: 3.0,
      },
      EnvSpec::ad(0.001, 0.25),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 69 cabasa
    69 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 6_500.0,
        res: 1.0,
      },
      EnvSpec::ad(0.001, 0.08),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 70 maracas
    70 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 5_500.0,
        res: 1.0,
      },
      EnvSpec::ad(0.001, 0.06),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 71 short whistle
    71 => drum(
      Some(DrumTone {
        wave: Square,
        freq: 2_400.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_500.0,
        res: 3.0,
      },
      EnvSpec::ad(0.005, 0.2),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.55,
    ),
    // 72 long whistle
    72 => drum(
      Some(DrumTone {
        wave: Square,
        freq: 2_400.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_500.0,
        res: 3.0,
      },
      EnvSpec::ad(0.005, 0.6),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.55,
    ),
    // 73 short guiro
    73 => drum(
      None,
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.005, 0.1),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 74 long guiro
    74 => drum(
      None,
      1.0,
      DrumFilter {
        mode: Band,
        cutoff: 2_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.01, 0.4),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 75 claves
    75 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 2_500.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 3.0,
      },
      EnvSpec::ad(0.001, 0.1),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 76 hi wood block
    76 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 1_500.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 2.5,
      },
      EnvSpec::ad(0.001, 0.12),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 77 lo wood block
    77 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 1_200.0,
        mix: 0.8,
      }),
      0.2,
      DrumFilter {
        mode: Band,
        cutoff: 3_500.0,
        res: 2.5,
      },
      EnvSpec::ad(0.001, 0.14),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.6,
    ),
    // 78 mute cuica
    78 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 600.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_000.0,
        res: 2.0,
      },
      EnvSpec::ad(0.005, 0.1),
      DrumSweep {
        from: 1.5,
        time: 0.05,
      },
      0.55,
    ),
    // 79 open cuica
    79 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 600.0,
        mix: 0.7,
      }),
      0.3,
      DrumFilter {
        mode: Band,
        cutoff: 2_000.0,
        res: 2.0,
      },
      EnvSpec::ad(0.005, 0.3),
      DrumSweep {
        from: 1.5,
        time: 0.05,
      },
      0.55,
    ),
    // 80 mute triangle
    80 => group(
      drum(
        Some(DrumTone {
          wave: Sine,
          freq: 4_500.0,
          mix: 0.9,
        }),
        0.1,
        DrumFilter {
          mode: Band,
          cutoff: 4_500.0,
          res: 4.0,
        },
        EnvSpec::ad(0.001, 0.1),
        DrumSweep {
          from: 1.0,
          time: 0.01,
        },
        0.55,
      ),
      2,
    ),
    // 81 open triangle
    81 => group(
      drum(
        Some(DrumTone {
          wave: Sine,
          freq: 4_500.0,
          mix: 0.9,
        }),
        0.1,
        DrumFilter {
          mode: Band,
          cutoff: 4_500.0,
          res: 4.0,
        },
        EnvSpec::ad(0.001, 0.8),
        DrumSweep {
          from: 1.0,
          time: 0.01,
        },
        0.55,
      ),
      2,
    ),
    // 82 shaker
    82 => drum(
      None,
      1.0,
      DrumFilter {
        mode: High,
        cutoff: 6_000.0,
        res: 1.0,
      },
      EnvSpec::ad(0.001, 0.08),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 83 jingle bell
    83 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 5_500.0,
        mix: 0.5,
      }),
      0.5,
      DrumFilter {
        mode: High,
        cutoff: 5_500.0,
        res: 2.0,
      },
      EnvSpec::ad(0.002, 0.4),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 84 belltree
    84 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 5_000.0,
        mix: 0.6,
      }),
      0.4,
      DrumFilter {
        mode: High,
        cutoff: 5_000.0,
        res: 2.0,
      },
      EnvSpec::ad(0.002, 0.6),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 85 castanets
    85 => drum(
      Some(DrumTone {
        wave: Triangle,
        freq: 1_400.0,
        mix: 0.6,
      }),
      0.4,
      DrumFilter {
        mode: Band,
        cutoff: 4_000.0,
        res: 3.0,
      },
      EnvSpec::ad(0.001, 0.08),
      DrumSweep {
        from: 1.0,
        time: 0.01,
      },
      0.5,
    ),
    // 86 mute surdo
    86 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 90.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 1_500.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.25),
      DrumSweep {
        from: 2.5,
        time: 0.04,
      },
      0.85,
    ),
    // 87 open surdo
    87 => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 90.0,
        mix: 0.9,
      }),
      0.3,
      DrumFilter {
        mode: Low,
        cutoff: 1_500.0,
        res: 0.8,
      },
      EnvSpec::ad(0.001, 0.5),
      DrumSweep {
        from: 2.5,
        time: 0.04,
      },
      0.85,
    ),

    _ => drum(
      Some(DrumTone {
        wave: Sine,
        freq: 200.0,
        mix: 0.5,
      }),
      0.5,
      DrumFilter {
        mode: Band,
        cutoff: 2_000.0,
        res: 1.0,
      },
      EnvSpec::ad(0.002, 0.15),
      DrumSweep {
        from: 1.5,
        time: 0.02,
      },
      0.6,
    ),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn standard_gm_keys_are_distinct_from_default() {
    // Spot-check that key drum keys produce specific patches.
    assert!(drum_patch(36).tone.unwrap().freq < 100.0); // bass drum is low
    assert_eq!(drum_patch(42).group, 1); // closed hi-hat is in choke group
    assert_eq!(drum_patch(46).group, 1); // open hi-hat is in choke group
  }
}
