mod args;
mod grep;

use std::io::{self, Write};
use std::process;

use args::Command;


fn main() -> ! {
  fn run() -> io::Result<()> {
    let command = args::parse().map_err(
      |e| {
        eprintln!("{}", e.message);
        io::ErrorKind::InvalidInput
      }
    )?;

    match command {
      Command::Help(msg) | Command::Version(msg) => writeln!(io::stdout(), "{}", msg),
      Command::Grep(args) => grep::run(args)
    }
  }

  process::exit(
    match run() {
      Ok(()) => 0,
      Err(e) => match e.kind() {
        io::ErrorKind::InvalidInput     => 3,
        io::ErrorKind::NotFound         => 4,
        io::ErrorKind::PermissionDenied => 5,
        io::ErrorKind::BrokenPipe       => 6,
        io::ErrorKind::Interrupted      => 130,
        _ => 2
      }
    }
  )
}
