# Instrument demos

One MIDI file per General MIDI program (`<program>-<slug>.mid`), each playing a
short phrase chosen to suit how that instrument is typically used: pianos
arpeggiate a chord progression, basses walk a groove, strings sing a legato
line, brass stabs a fanfare, pads sustain, sound-effect "instruments" just
trigger the sound a few times, and so on. The phrase is shared within a GM
family, so the program (timbre) is what differs from file to file.

Both the `.mid` source and a compressed `.opus` render of each demo are tracked.
Opus is used because it is a royalty-free open standard (libopus is BSD) and
gives the smallest files — the whole set is a few MB rather than ~180 MB of WAV.

To regenerate the MIDIs with the current phrase definitions (`src/demo.rs`):

```sh
make instruments
```

To re-render every demo to `.opus` with the current synth (renders each `.mid`
to a temporary WAV, encodes it to mono Opus with `ffmpeg`, and removes the WAV):

```sh
make examples
```

Write or render a single instrument directly:

```sh
rumpl demo 56 .            # writes 056-trumpet.mid here
rumpl render 056-trumpet.mid 056-trumpet.wav
rumpl play 056-trumpet.mid
```
