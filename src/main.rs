use {
  arguments::Arguments,
  cargo_metadata::{
    CargoOpt, Metadata, MetadataCommand, PackageId,
    camino::{Utf8Path, Utf8PathBuf},
    semver::Version,
  },
  clap::{Args, Parser, builder::styling},
  error::Error,
  path::Path,
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu, ensure},
  std::{
    collections::{HashMap, VecDeque},
    io::{self, IsTerminal},
    process::ExitCode,
  },
};

mod arguments;
mod error;
mod path;

fn main() -> ExitCode {
  if let Err(error) = run() {
    if io::stderr().is_terminal() {
      eprintln!("\x1b[1;31merror\x1b[0m: \x1b[1m{error}\x1b[0m");
    } else {
      eprintln!("error: {error}");
    }

    let causes = error.iter_chain().skip(1).count();

    for (i, source) in error.iter_chain().skip(1).enumerate() {
      eprintln!(
        "       {}─ {source}",
        if i < causes - 1 { '├' } else { '└' }
      );
    }

    ExitCode::FAILURE
  } else {
    ExitCode::SUCCESS
  }
}

fn run() -> Result<(), Error> {
  let Arguments::Path(path) = Arguments::parse();
  path.run()
}
