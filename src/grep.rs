use std::io;
use std::io::{Read, Write};
use std::fs::File;

use regex::bytes::{Regex, RegexBuilder};

use crate::args::{self, Args};


fn build_pattern(
  pattern: &String,
  options: &args::Options
) -> Result<Regex, regex::Error> {
  let mut builder = RegexBuilder::new(pattern);

  builder.unicode(false);
  builder.dot_matches_new_line(true);
  builder.case_insensitive(options.case_insensitive);

  builder.build()
}


fn grep_filename(
  stdout: &mut io::StdoutLock,
  options: &args::Options,
  path: &str,
  pattern: &Regex,
  buffer: &[u8]
) -> io::Result<()> {
  if pattern.is_match(buffer) ^ options.inverse {
    writeln!(stdout, "{}", path)?;
  }

  Ok(())
}


fn grep_bytes(
  stdout: &mut io::StdoutLock,
  options: &args::Options,
  pattern: &Regex,
  buffer: &[u8]
) -> io::Result<()> {
  if options.inverse {
    for m in pattern.split(buffer) {
      stdout.write(m)?;
      writeln!(stdout)?;
    }
  }
  else {
    for m in pattern.find_iter(buffer) {
      stdout.write(m.as_bytes())?;
      writeln!(stdout)?;
    }
  };

  Ok(())
}


fn grep_position(
  stdout: &mut io::StdoutLock,
  options: &args::Options,
  pattern: &Regex,
  buffer: &[u8]
) -> io::Result<()> {
  let mut write_hex = |x| writeln!(stdout, "0x{:x}", x);

  if options.inverse {
    let mut last: usize = 0; // Start from the beginning of the buffer.

    for m in pattern.find_iter(buffer) {
      for offset in last .. m.start() { // Print each offset inside the span.
        write_hex(offset)?;
      }

      last = m.end()
    }

    for offset in last .. buffer.len() { // Print the last span, if any.
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


pub fn run(args: Args) -> io::Result<()> {
  let Args { options, pattern, files } = args;


  let pattern = build_pattern(&pattern, &options).map_err(
    |e| {
      eprintln!("Error: invalid pattern '{}', {}", pattern, e);
      io::ErrorKind::InvalidInput
    }
  )?;


  let stdout = io::stdout();
  let mut stdout = stdout.lock();

  let mut buffer = Vec::<u8>::new();

  files.into_iter().fold(
    Ok(()),
    |result, path| {
      buffer.clear();

      let (read_result, path) = if path == "-" {
        (io::stdin().lock().read_to_end(&mut buffer), "<stdin>")
      }
      else {
        let mut file = File::open(&path)
                            .map_err(|e| {
                              eprintln!("Error: failed to open file '{}'", path);
                              e
                            })?;

        // Resize buffer to the file size if it exceeds the current size:
        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0) as usize;
        buffer.reserve(file_size.saturating_sub(buffer.len()));

        (file.read_to_end(&mut buffer), path.as_str())
      };


      read_result.map_err(
        |e| {
          eprintln!("Error: failed to read file '{}'", path);
          e
        }
      )?;


      match options.output {
        args::Output::Filename => grep_filename(&mut stdout, &options, &path, &pattern, &buffer),
        args::Output::Bytes    => grep_bytes(&mut stdout, &options, &pattern, &buffer),
        args::Output::Position => grep_position(&mut stdout, &options, &pattern, &buffer)
      }?;


      result
    }
  )
}
