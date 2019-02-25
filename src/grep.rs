use std::io;
use std::io::{Read, Write};
use std::fs::File;

use regex::bytes::{Regex, RegexBuilder};

use crate::args::{Args, GrepOutput};



fn build_pattern(pattern: &String) -> Result<Regex, regex::Error> {
  let mut builder = RegexBuilder::new(pattern);
  builder.unicode(false);
  builder.dot_matches_new_line(true);

  builder.build()
}


fn grep_filename(
  stdout: &mut io::StdoutLock,
  args: &Args,
  path: &String,
  pattern: &Regex,
  buffer: &[u8]
) -> Result<(), io::Error> {
  if pattern.is_match(buffer) ^ args.inverse {
    writeln!(stdout, "{}", path)?;
  }

  Ok(())
}


fn grep_bytes(
  stdout: &mut io::StdoutLock,
  args: &Args,
  pattern: &Regex,
  buffer: &[u8]
) -> Result<(), io::Error> {
  let matches: Box<Iterator<Item = &[u8]>> = if args.inverse {
    Box::new(pattern.split(buffer))
  }
  else {
    Box::new(pattern.find_iter(buffer).map(|m| m.as_bytes()))
  };

  for m in matches {
    stdout.write(m)?;
    writeln!(stdout)?;
  }

  Ok(())
}


fn grep_position(
  stdout: &mut io::StdoutLock,
  args: &Args,
  pattern: &Regex,
  buffer: &[u8]
) -> Result<(), io::Error> {
  let mut write_hex = |x| writeln!(stdout, "0x{:x}", x);

  if args.inverse {
    let mut last: usize = 0;

    for m in pattern.find_iter(buffer) {
      for offset in last .. m.start() { // print each offset inside the span.
        write_hex(offset)?;
      }

      last = m.end()
    }

    for offset in last .. buffer.len() { // print the last span, if any.
      write_hex(offset)?;
    }
  }
  else {
    for m in pattern.find_iter(buffer) {
      write_hex(m.start())?;
    }
  }

  Ok(())
}


pub fn run(args: &Args) -> Result<(), io::Error> {
  let pattern = build_pattern(&args.pattern).map_err(
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

      // Resize buffer to the file size if it exceeds the current size:
      let file_size = file.metadata().map(|m| m.len()).unwrap_or(0) as usize;
      buffer.reserve(file_size.saturating_sub(buffer.len()));

      match file.read_to_end(&mut buffer) {
        Ok(_)  => (),
        Err(e) => return {
          eprintln!("Error: failed to read file '{}'", path);
          Err(e)
        }
      };


      match args.output {
        GrepOutput::Filename => grep_filename(&mut stdout, args, &path, &pattern, &buffer),
        GrepOutput::Bytes    => grep_bytes(&mut stdout, args, &pattern, &buffer),
        GrepOutput::Position => grep_position(&mut stdout, args, &pattern, &buffer)
      }?;


      result
    }
  )
}
