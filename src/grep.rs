use std::io;
use std::io::{Read, Write};
use std::fs::File;

use regex::bytes::RegexBuilder;

use crate::args::{Args, GrepOutput};



pub fn grep(args: &Args) -> Result<(), io::Error> {
  let mut builder = RegexBuilder::new(&args.pattern);
  builder.unicode(false);

  let pattern = builder.build().map_err(
    |e| {
      eprintln!("Error: invalid pattern '{}', {}", args.pattern, e);
      io::ErrorKind::InvalidInput
    }
  )?;


  let stdout = io::stdout();
  let mut stdout = stdout.lock();

  let mut buffer = Vec::<u8>::new();

  args.files.iter().fold(
    Ok(()),
    |result, path| {
      buffer.clear();

      let mut file = match File::open(&path) {
        Ok(f)  => f,
        Err(e) => return {
          eprintln!("Error: failed to open file '{}'", path);
          Err(e)
        }
      };

      match file.read_to_end(&mut buffer) {
        Ok(_)  => (),
        Err(e) => return {
          eprintln!("Error: failed to read file '{}'", path);
          Err(e)
        }
      };

      match args.output {
        GrepOutput::Filename => {
          if pattern.is_match(&buffer) {
            writeln!(stdout, "{}", path)?;
          }
        },
        GrepOutput::Bytes => {
          for match_ in pattern.find_iter(&buffer) {
            stdout.write(match_.as_bytes())?;
            writeln!(stdout)?;
          }
        },
        GrepOutput::Position => {
          for match_ in pattern.find_iter(&buffer) {
            writeln!(stdout, "0x{:x}", match_.start())?;
          }
        }
      };


      result
    }
  )
}
