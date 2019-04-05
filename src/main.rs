mod args;
mod grep;

use std::env;
use std::io::{self, Write};
use std::process;

use args::Command;


/// Main does not return because we use `process::exit`.
fn main() -> ! {
  /// Run bgrep with `std::env::args_os`, outputting to stdout.
  /// Error detail may be outputted to stderr.
  /// Returns whether there was a match.
  fn run() -> io::Result<bool> { // Returns whether there was a match.
    let args = env::args_os();

    let command = args::parse(args).map_err(
      |e| {
        eprintln!("{}", e.message);
        io::ErrorKind::InvalidInput
      }
    )?;


    // Lock stdout to prevent repetitive locking.
    let stdout = io::stdout();
    let mut stdout = stdout.lock();


    match command {
      Command::Grep(args) => grep::run(args, &mut stdout),
      Command::Help(msg) | Command::Version(msg) => {
        writeln!(stdout, "{}", msg)?;
        Ok(true)
      }
    }
  }

  process::exit(
    match run() {
      Ok(true)  => 0, // There was at least one match.
      Ok(false) => 1, // There was no match.
      Err(e) => match e.kind() {
        io::ErrorKind::InvalidInput     => 3,
        io::ErrorKind::NotFound         => 4,
        io::ErrorKind::PermissionDenied => 5,
        io::ErrorKind::Interrupted      => 130,
        _ => 2
      }
    }
  )
}
