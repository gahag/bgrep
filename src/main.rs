extern crate regex;
extern crate getopts;


mod args;
mod grep;


use std::io;
use std::process;



fn main() -> ! {
  fn run() -> Result<(), io::Error> {
    let args = args::parse()?;

    if let Some(help) = args.help {
      eprint!("{}", help);
      return Ok(())
    }

    grep::run(&args)
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
