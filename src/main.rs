mod cli;

use anyhow::Result;
use clap::Parser;
use rumpl::{midi, output};

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

fn warn_soundfont(p: Option<&std::path::Path>) {
  if p.is_some() {
    eprintln!(
      "warning: --soundfont is ignored; rumpl synthesizes voices algorithmically"
    );
  }
}
