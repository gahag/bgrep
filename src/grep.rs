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
) -> io::Result<bool> {
  if options.inverse {
    // if the pattern matches multiple times, comprising the entire buffer, then no
    // inverse match is present.
    let mut matches = pattern.find_iter(buffer);

    let mut end = 0; // Start from the beginning of the buffer.

    let inverse_match = matches.find(
      |m| {
        let matched = m.start() > end;

        end = m.end();

        matched
      }
    );

    let matched = (inverse_match.is_some() || end < buffer.len()) // Check the last slice.
                ^ options.non_matching;

    if matched {
      writeln!(stdout, "{}", path)?;
    }

    Ok(matched)
  }
  else {
    let matched = pattern.is_match(buffer) ^ options.non_matching;

    if matched {
      writeln!(stdout, "{}", path)?;
    }

    Ok(matched)
  }
}


fn grep_bytes(
  stdout: &mut io::StdoutLock,
  options: &args::Options,
  pattern: &Regex,
  buffer: &[u8]
) -> io::Result<bool> {
  let mut write_bytes = |bs| {
    stdout.write(bs)?;
    writeln!(stdout)
  };


  let mut matched = false;

  if options.inverse {
    let mut matches = pattern.split(buffer);

    if let Some(bs) = matches.next() {
      if !bs.is_empty() { // A regex may have a empty match, but when inverse matching 
        write_bytes(bs)?; // we disconsider empty intervals.
        matched = true;
      }
    };

    for bs in matches {
      if !bs.is_empty() {
        write_bytes(bs)?;
      }
    }
  }
  else {
    let mut matches = pattern.find_iter(buffer);

    if let Some(m) = matches.next() {
      write_bytes(m.as_bytes())?;
      matched = true;
    }

    for m in matches {
      write_bytes(m.as_bytes())?;
    }
  };


  Ok(matched)
}


fn grep_offset(
  stdout: &mut io::StdoutLock,
  options: &args::Options,
  pattern: &Regex,
  buffer: &[u8]
) -> io::Result<bool> {
  let mut write_hex = |x| writeln!(stdout, "0x{:x}", x);


  let mut matches = pattern.find_iter(buffer);

  let mut matched = false;

  if options.inverse {
    // if the pattern matches multiple times, comprising the entire buffer, then no
    // inverse match is present.
    let mut end = 0; // Start from the beginning of the buffer.

    for m in matches {
      if m.start() > end {
        write_hex(end)?;
        matched = true;
      }

      end = m.end()
    }

    if end < buffer.len() { // Write the last span, if any.
      write_hex(end)?;
      matched = true;
    }
  }
  else {
    if let Some(m) = matches.next() {
      write_hex(m.start())?;
      matched = true;
    }

    for m in matches {
      write_hex(m.start())?;
    }
  }


  Ok(matched)
}


pub fn run(args: Args) -> io::Result<bool> {
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
    Ok(false), // : io::Result<bool>, whether there was a match.
    |result, path| {
      buffer.clear();

      let (read_result, path) =
        if path == "-" {
          (io::stdin().lock().read_to_end(&mut buffer), "<stdin>")
        }
        else {
          let mut file = File::open(&path)
                              .map_err(|e| {
                                eprintln!("Error: failed to open file '{}'", path);
                                e
                              })?;

          // Resize buffer to the file size if it exceeds the current size:
          let file_size = file.metadata()
                              .map(|m| m.len())
                              .unwrap_or(0) as usize;
          buffer.reserve(
            file_size.saturating_sub(buffer.len())
          );

          (file.read_to_end(&mut buffer), path.as_str())
        };


      if let Err(e) = read_result {
        eprintln!("Error: failed to read file '{}'", path);
        return Err(e);
      }


      let buffer = match (options.trim_ending_newline, buffer.last()) {
        (true, Some(b'\n')) => &buffer[.. buffer.len() - 1],
        _ => &buffer
      };


      let matched = match options.output {
        args::Output::FileName => grep_filename(&mut stdout, &options, &path, &pattern, buffer),
        args::Output::Bytes    => grep_bytes(&mut stdout, &options, &pattern, buffer),
        args::Output::Offset   => grep_offset(&mut stdout, &options, &pattern, buffer)
      }?;


      if matched {
        result.and(Ok(true))
      }
      else {
        result
      }
    }
  )
}
