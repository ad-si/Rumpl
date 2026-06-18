mod cli;

use anyhow::Result;
use clap::Parser;
use rumpl::synth::patches::gm_program_name;
use rumpl::{demo, midi, output};

fn main() -> Result<()> {
  let args = cli::Cli::parse();
  match args.command {
    cli::Cmd::Play {
      file,
      gain,
      tail,
      soundfont,
    } => {
      warn_soundfont(soundfont.as_deref());
      let seq = midi::load(&file)?;
      output::play::play(seq, gain, tail)?;
    }
    cli::Cmd::Info { file } => {
      let info = midi::info(&file)?;
      print_info(&file, &info);
    }
    cli::Cmd::Instruments => {
      for program in 0..128u8 {
        println!("{:>3}  {}", program, gm_program_name(program));
      }
    }
    cli::Cmd::Demo { program, out } => {
      let out = if out.is_dir() {
        out.join(demo::demo_filename(program))
      } else {
        out
      };
      demo::write_demo_midi(program, &out)?;
      eprintln!(
        "wrote demo of {} ({program}) → {}",
        gm_program_name(program),
        out.display()
      );
    }
    cli::Cmd::Render {
      file,
      out,
      gain,
      sample_rate,
      tail,
      soundfont,
    } => {
      warn_soundfont(soundfont.as_deref());
      let seq = midi::load(&file)?;
      output::render::render_to_wav(&seq, &out, sample_rate, gain, tail)?;
      eprintln!("rendered {} → {}", file.display(), out.display());
    }
  }
  Ok(())
}

fn print_info(file: &std::path::Path, info: &midi::FileInfo) {
  let mins = (info.duration / 60.0).floor() as u64;
  let secs = info.duration - (mins as f64) * 60.0;

  println!("{}", file.display());
  println!("  format:   {}", info.format);
  println!("  tracks:   {}", info.track_count);
  match info.ppq {
    Some(ppq) => println!("  division: {ppq} PPQ"),
    None => println!("  division: SMPTE timecode"),
  }
  println!("  length:   {mins}:{secs:05.2} ({:.2} s)", info.duration);

  let tempos: Vec<String> =
    info.tempos_bpm.iter().map(|b| format!("{b:.1}")).collect();
  let label = if info.tempos_bpm.len() > 1 {
    "tempos"
  } else {
    "tempo"
  };
  println!("  {label}:    {} BPM", tempos.join(", "));
  println!("  notes:    {}", info.note_count);

  println!("  voices:");
  for ch in &info.channels {
    let name = if ch.is_drums {
      "Drum Kit"
    } else {
      gm_program_name(ch.program)
    };
    println!(
      "    ch {:>2}: {name} ({} notes)",
      ch.channel + 1,
      ch.note_count
    );
  }
}

fn warn_soundfont(p: Option<&std::path::Path>) {
  if p.is_some() {
    eprintln!(
      "warning: --soundfont is ignored; rumpl synthesizes voices algorithmically"
    );
  }
}
