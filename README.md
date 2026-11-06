# Rumpl

Sample-free General MIDI player and renderer. Every voice is generated
algorithmically — no SoundFont, no sample library, no FluidSynth.
Expect a chiptune / analog-synth aesthetic.


## Install

```sh
cargo install rumpl
```

Or from a local checkout:

```sh
cargo install --path .
```


## Usage

```sh
rumpl play   file.mid               # real-time playback
rumpl render file.mid               # render to file.wav alongside the input
rumpl render file.mid out.wav       # offline render to 44.1 kHz / 16-bit stereo WAV
```

Flags:

- `--gain <f32>` — master gain (default `0.8`)
- `--sample-rate <u32>` — render sample rate (default `44100`)
- `--tail <f32>` — render seconds of release tail after last note-off (default `3.0`)


## How it works

- `midly` parses the SMF; events are merged across tracks and stamped with
  absolute sample indices via the tempo map.
- A 64-voice synth maps each MIDI program to a static `Patch` (oscillator stack,
  ADSR, filter, LFO). Channel 10 uses a separate per-key drum patch table.
- Output goes to `cpal` for `play` or `hound` for `render`.
