mod args;
mod grep;

use std::io::{self, Write};
use std::process;

use args::Command;


fn main() -> ! {
  fn run() -> io::Result<bool> { // Returns whether there was a match.
    let command = args::parse().map_err(
      |e| {
        eprintln!("{}", e.message);
        io::ErrorKind::InvalidInput
      }
    )?;

    match command {
      Command::Grep(args) => grep::run(args),
      Command::Help(msg) | Command::Version(msg) => {
        writeln!(io::stdout(), "{}", msg)?;
        Ok(true)
      }
    }
  }

  process::exit(
    match run() {
      Ok(true)  => 0,
      Ok(false) => 1,
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
