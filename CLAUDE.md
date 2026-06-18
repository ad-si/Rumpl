# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`rumpl` is a sample-free General MIDI player and renderer written in Rust. There is no SoundFont or sample library: every voice is generated algorithmically from oscillator stacks, ADSR envelopes, filters, LFOs, and 2-op FM. The aesthetic is chiptune / analog-synth, not Sound Canvas. The `--soundfont` flag exists for compatibility but is intentionally ignored.

## Common commands

Use the makefile targets — they are the canonical entry points.

- `make build` — `cargo build --release`
- `make test` — `cargo test -- --show-output` (runs unit tests in `src/` and the `render_smoke` integration test, which shells out to the built binary via `CARGO_BIN_EXE_rumpl`)
- `make format` — `cargo clippy --fix` + `cargo fmt`
- `make install` — `cargo install --path .`

Run a single test: `cargo test <name> -- --show-output` (e.g. `cargo test honours_set_tempo_at_tick_zero`).

Run the binary directly: `cargo run --release -- play file.mid` or `cargo run --release -- render file.mid out.wav`.

## Architecture

The pipeline is a strict three-stage pull: parse → sequence → synth → output.

### `midi.rs` — parse
`midly` decodes the SMF. The parser walks every track to build a tempo map of `(absolute_tick, microseconds_per_quarter)` pairs, deduped per tick (later overrides earlier). Default 120 BPM is only inserted if no tempo event lands on tick 0. SMPTE timecode files are coerced into a quasi-PPQ. A second pass merges all tracks into a flat `Vec<TimedEvent>` sorted by absolute tick, then converts to seconds via the tempo map. The result is a `Sequence { events, duration }` indexed by seconds, not ticks — the synth never sees tempo again.

### `sequencer.rs` — drive synth in fixed blocks
`Sequencer::render_block` walks through `seq.events` and the sample clock together. For each block it splits the block at every event boundary, calls `Synth::render` on each subspan, and dispatches the event in between. Events are converted to sample positions via `time * sr`. There is no internal scheduling drift across blocks; the sample clock is the source of truth.

### `synth/` — voice manager and DSP
- `Synth` owns 16 `Channel`s and up to `MAX_VOICES = 64` `Voice`s. Voice stealing prefers the oldest released voice, falling back to the oldest overall.
- `Channel` holds per-channel MIDI state: program, CC7/11 volume/expression, pan, mod wheel, sustain, pitch bend, RPN selector. Channel index 9 is the GM drum channel (`is_drums`).
- `Voice` is a tagged union: `VoiceKind::Tonal` (oscillator stack ± 2-op FM ± filter ± LFO), `VoiceKind::Drum` (tone+noise + downward pitch sweep + filter), or `VoiceKind::Plucked` (one or more Karplus-Strong `KsString`s). All share an `Env`. For plucked voices the env is only a note-on gate plus a note-off damper — the actual pitched decay comes from the string loop, and idle detection additionally watches a per-voice energy follower so a held string is retired once it rings out.
- `KsString` (in `voice.rs`) is the physical-modelled plucked string: a delay line excited at note-on with a triangular initial displacement peaking at the plucking point (modal spectrum sin(n·π·p)/n² — strong fundamental, gentle roll-off, pluck-position comb built in) plus a little noise for the plectrum chiff, velocity-low-passed for dynamics and DC-removed; fed back through a two-tap loop low-pass (`damp` = brightness/decay-of-highs) and a first-order tuning all-pass (fractional-sample delay for exact pitch). `decay_gain` sets the feedback so the fundamental falls ~60 dB in the requested ring time; `key_decay_scale` shortens that ring for higher notes. Live pitch bend is not tracked (delay length is fixed at note-on).
- `patches.rs` maps the 128 GM programs to a static `[Patch; 128]` table built `const`, indexed by `gm_patch(program)`. Edit ranges of programs together — the table is dense, not sparse.
- `drums.rs` maps GM drum note numbers (35..81) to `DrumPatch`s. Drums use mute groups (e.g. open/closed hi-hats choke each other) via the non-zero `group` field — `note_on` on a drum channel scans existing voices in the same group and releases them.
- `osc.rs` / `env.rs` / `filter.rs` are the small DSP primitives: `Wave` (sine/triangle/saw/square/noise), `Env` (ADSR with idle detection), `Svf` (state-variable filter, lowpass/highpass/bandpass).
- LFOs target one of `Pitch` / `Filter` / `Amp` and have a per-voice ramp-in delay. Pitch-bend reads `Channel::pitch_bend` per sample inside the voice render loop, so it tracks live without re-keying.

### `output/` — sinks
- `render.rs`: blocking pull through `Sequencer` for `total_samples` frames, each block written as 16-bit interleaved stereo via `hound`. Block size 512 frames.
- `play.rs`: producer thread renders into a `ringbuf` `HeapRb<f32>` (~185 ms at 44.1 kHz); `cpal` audio callback drains it. Block size 256 frames. Handles f32/i16/u16 device formats and downmixes/fans-out to non-stereo devices. Uses an `AtomicBool` stop flag for shutdown coordination.

## Adding or tweaking sounds

- New tonal program: add a match arm in `patch_for` in `src/synth/patches.rs`. Reuse the existing `OscSpec` constants (`SAW`, `STRING_OSC`, `PAD_OSC`, …) where possible. The `every_program_has_a_patch` test enforces non-empty oscillators and non-zero gain for all 128 slots.
- New drum: add a match arm in `drum_patch` in `src/synth/drums.rs`. Use `group(...)` to wrap a `DrumPatch` for choke behavior.
- New plucked-string program (guitar, harp, pizzicato, …): give the patch a `PluckSpec` via the `plucked(...)` helper in `patches.rs` (see `PLUCK_HARPSI`). The patch's `oscs`/`env` then only serve as an inert fallback that satisfies `every_program_has_a_patch`; `note_on` routes any patch with `pluck: Some(_)` to `Voice::new_plucked`. Tune the timbre with `damp` (brightness), `pluck_pos` (comb colour), `decay` (ring time), and the per-rank `StringSpec`s.
- New oscillator waveform or filter mode: extend the `Wave` / `FilterMode` enum and the corresponding `tick`/processing in `osc.rs` / `filter.rs`, then thread it through `OscSpec` / `FilterSpec`.

## Conventions

- Rust edition 2024.
- Nothing depends on a SoundFont, sample file, or external audio asset. Don't add one.
- The render pipeline is deterministic given (sequence, sample rate, gain, tail). The smoke test in `tests/render_smoke.rs` relies on this — peak and frame count are asserted within tolerances.
- Use `Edit`/`Read` over `sed`/`cat`. Use the project search tool over shelling to `grep`.
