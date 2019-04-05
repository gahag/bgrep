use std::io;
use std::io::{Read, Write};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::fmt::Display;

use regex::bytes::{Regex, RegexBuilder};

use crate::args::{self, Args};


/// Build the regex pattern with the given options.
/// By default, the `unicode` flag is set to false, and `dot_matches_new_line` set to true.
fn build_pattern<P: AsRef<str>>(
  pattern: &P,
  options: &args::Options
) -> Result<Regex, regex::Error> {
  let mut builder = RegexBuilder::new(pattern.as_ref());

  builder.unicode(false);
  builder.dot_matches_new_line(true);
  builder.case_insensitive(options.case_insensitive);

  builder.build()
}


/// Run bgrep, outputting `path` to the given `out` if there is a match.
/// Returns whether there was a match.
fn grep_filename<O: Write, P: Display, B: AsRef<[u8]>>(
  out: &mut O,
  options: &args::Options,
  pattern: &Regex,
  path: P,
  buffer: B
) -> io::Result<bool> {
  let buffer = buffer.as_ref();

  // When inverse matching, matches must be checked until a "hole" is found.
  // Otherwise, the more performant `Regex::is_match` can be used.
  if options.inverse {
    // if the pattern matches multiple times, comprising the entire buffer, then no
    // inverse match is present.
    let mut matches = pattern.find_iter(buffer);

    let mut end = 0; // Start from the beginning of the buffer.

    // Try to find a "hole" between matches:
    let inverse_match = matches.find(
      |m| {
        let matched = m.start() > end;

        end = m.end();

        matched
      }
    );

    // Also check for a "hole" after the last match.
    let matched = (inverse_match.is_some() || end < buffer.len())
                ^ options.non_matching; // List non matching files.

    if matched {
      writeln!(out, "{}", path)?;
    }

    Ok(matched)
  }
  else {
    let matched = pattern.is_match(buffer) ^ options.non_matching;

    if matched {
      writeln!(out, "{}", path)?;
    }

    Ok(matched)
  }
}


/// Run bgrep, outputting the matched bytes to the given `out`.
/// Returns whether there was a match.
fn grep_bytes<O: Write, P: Display, B: AsRef<[u8]>>(
  out: &mut O,
  options: &args::Options,
  pattern: &Regex,
  path: P,
  buffer: B,
) -> io::Result<bool> {
  let buffer = buffer.as_ref();

  let mut write_bytes = |bs| {
    if options.print_filename {
      write!(out, "{}: ", path)?;
    }

    out.write_all(bs)?;
    writeln!(out)
  };


  let mut matched = false;

  if options.inverse {
    // `Regex::split` yields the slices outside the matches.
    let mut matches = pattern.split(buffer);

    // Set `matched` if there is a first occurrence:
    if let Some(bs) = matches.next() {
      if !bs.is_empty() { // A regex may have a empty match, but when inverse matching
        write_bytes(bs)?; // we disconsider empty intervals.
        matched = true;
      }
    };

    // Iterate the remaining matches:
    for bs in matches {
      if !bs.is_empty() {
        write_bytes(bs)?;
      }
    }
  }
  else {
    let mut matches = pattern.find_iter(buffer);

    // Set `matched` if there is a first occurrence:
    if let Some(m) = matches.next() {
      write_bytes(m.as_bytes())?;
      matched = true;
    }

    // Iterate the remaining matches:
    for m in matches {
      write_bytes(m.as_bytes())?;
    }
  };


  Ok(matched)
}


/// Run bgrep, outputting the matche's offset in hex to the given `out`.
/// Returns whether there was a match.
fn grep_offset<O: Write, P: Display, B: AsRef<[u8]>>(
  out: &mut O,
  options: &args::Options,
  pattern: &Regex,
  path: P,
  buffer: B
) -> io::Result<bool> {
  let buffer = buffer.as_ref();

  let mut write_hex = |x| {
    if options.print_filename {
      writeln!(out, "{}: 0x{:x}", path, x)
    } else {
      writeln!(out, "0x{:x}", x)
    }
  };


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

    if end < buffer.len() { // Also check for a "hole" after the last match.
      write_hex(end)?;
      matched = true;
    }
  }
  else {
    // Set `matched` if there is a first occurrence:
    if let Some(m) = matches.next() {
      write_hex(m.start())?;
      matched = true;
    }

    // Iterate the remaining matches:
    for m in matches {
      write_hex(m.start())?;
    }
  }


  Ok(matched)
}


/// Run bgrep with the given options, outputting to the given `out`.
/// Error detail may be outputted to stderr.
/// Returns whether there was a match.
fn run_file<O: Write, P: AsRef<Path>, B: AsMut<Vec<u8>>>(
  out: &mut O,
  options: &args::Options,
  pattern: &Regex,
  path: P,
  buffer: &mut B
) -> io::Result<bool> {
  let buffer = buffer.as_mut();
  let path = path.as_ref();

  buffer.clear();

  let (read_result, path) =
    if path == Path::new(args::STDIN) { // Path::new is cost-free.
      (io::stdin().lock().read_to_end(buffer), Path::new("<stdin>").display())
    }
    else {
      let mut file = File::open(path)
                          .map_err(|e| {
                            eprintln!("Error: failed to open file '{}'", path.display());
                            e
                          })?;

      // Resize buffer to the file size if it exceeds the current size.
      // Currently, the strategy is to grow if needed, and otherwise do nothing.
      // Considering we never shrink the buffer, this can be bad if the first file
      // is huge and the others are small.
      let file_size = file.metadata()
                          .map(|m| m.len())
                          .unwrap_or(0) as usize;
      buffer.reserve(
        file_size.saturating_sub(buffer.len())
      );

      (file.read_to_end(buffer), path.display())
    };

  if let Err(e) = read_result {
    eprintln!("Error: failed to read file '{}'", path);
    return Err(e);
  }


  // Trim the ending newline if requested and present:
  if options.trim_ending_newline && buffer.last() == Some(&b'\n') {
    buffer.pop();
  };


  let matched = match options.output {
    args::Output::FileName => grep_filename (out, options, pattern, path, buffer),
    args::Output::Bytes    => grep_bytes    (out, options, pattern, path, buffer),
    args::Output::Offset   => grep_offset   (out, options, pattern, path, buffer)
  }?;

  Ok(matched)
}

/// Run bgrep with the given args, outputting to stdout.
/// Error detail may be outputted to stderr.
/// Returns whether there was a match.
pub fn run<O: Write>(args: Args, out: &mut O) -> io::Result<bool> {
  // Deconstruct to split ownership:
  let Args { options, pattern, files } = args;


  let pattern = build_pattern(&pattern, &options).map_err(
    |e| {
      eprintln!("Error: invalid pattern '{}', {}", pattern, e);
      io::ErrorKind::InvalidInput
    }
  )?;


  // Reuse the same buffer for all the files, minimizing allocations.
  let mut buffer = Vec::<u8>::new();

  // The next part is a bit complicated:
  // Bgrep must return:
  //
  // 0 if there was any match, and no errors. `BrokenPipe` is not considered an error,
  //   but a signal to stop processing.
  //
  // 1 if there was no match, and no errors. `BrokenPipe` cannot happen in this case,
  //   because it should only happen when outputting, and no matches means no output.
  //
  // An error code corresponding to the last error. Common errors are `NotFound` and
  // `PermissionDenied`.

  // We need to store the last generated error if any, or whether there was a match:
  let mut result = Ok(false);

  // Converting to vec to use the owned iterator. Box<[T]> has no owned iterator.
  for file in files.to_vec() {
    let file: PathBuf = file; // Make sure we are using an owned iterator.

    match run_file(out, &options, &pattern, &file, &mut buffer) {
      Ok(false) => (),
      Ok(true) => result = result.map(|_| true), // Set to true if there was no error.
      Err(e) =>
        if e.kind() == io::ErrorKind::BrokenPipe {
          // Bail early on `BronkenPipe`, conserving the previous error if any.
          result = result.map(|_| true); // `BrokenPipe` only happens when outputting,
          break;                         // and that means there was a match.
        } else {
          result = Err(e) // Store the error and move on.
        }
    }
  }

  result
}
