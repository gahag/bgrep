use std::ffi::OsString;
use std::path::PathBuf;

use clap::{self, App, Arg, ArgMatches};
use clap::{crate_authors, crate_version, crate_name, crate_description};


/// The output mode.
#[derive(Debug)]
pub enum Output {
  FileName,
  Bytes,
  Offset
}

impl Default for Output {
  fn default() -> Output { Output::FileName }
}


/// The values of all flags, except help and version.
#[derive(Default, Debug)]
pub struct Options {
  pub inverse: bool,
  pub case_insensitive: bool,
  pub trim_ending_newline: bool,
  pub non_matching: bool, // Whether to print non matching files. Only true when (-L).
  pub print_filename: bool,
  pub output: Output
}


/// The arguments when the action is grep.
#[derive(Default, Debug)]
pub struct Args {
  pub options: Options,
  pub pattern: String,
  pub files: Box<[PathBuf]>
}


/// The action to be executed.
#[derive(Debug)]
pub enum Command {
  Help(String),
  Version(String),
  Grep(Args)
}


/// The error type for the argument parser. Contains only the error message.
#[derive(Debug)]
pub struct Error {
  pub message: String
}



/// The path used to denote reading from stdin.
pub const STDIN: &str = "-";



/// Build clap's `App`. This specifies all arguments and metadata.
fn build_app() -> App<'static, 'static> {
  App::new(crate_name!())
    .about(crate_description!())
    .author(crate_authors!())
    .version(crate_version!())
    .template("{bin} {version}\nMade by {author}\n{about}\n\n{usage}\n\nFLAGS:\n{flags}")
    // Positional arguments:
    .arg(
      Arg::with_name("pattern")
          .required(true)
          .index(1)
    )
    .arg(
      Arg::with_name("files")
        .multiple(true)
        .default_value(STDIN)
        .index(2)
    )
    // Matching flags:
    .arg(
      Arg::with_name("invert-match")
        .short("v")
        .long("invert-match")
        .help("Invert the sense of matching, to select non matching slices")
    )
    .arg(
      Arg::with_name("ignore-case")
        .short("i")
        .long("ignore-case")
        .help("Case insensitive matching for ASCII alphabetic characters")
    )
    // Input flags:
    .arg(
      Arg::with_name("trim-ending-newline")
        .short("n")
        .long("trim-ending-newline")
        .help("If the file ends with a newline, disconsider the last byte")
    )
    // Output flags:
    .arg(
      Arg::with_name("with-filename")
        .short("H")
        .long("with-filename")
        .help("Print the file name for each match (default when there are multiple files).")
        .overrides_with("no-filename")
    )
    .arg(
      Arg::with_name("no-filename")
        .short("h")
        .long("no-filename")
        .help("Suppress the file names on output (default when there is a single file).")
        .overrides_with("with-filename")
        .conflicts_with_all(&[
          "files-with-matches",
          "files-without-matches",
        ])
    )
    .arg(
      Arg::with_name("only-matching")
        .short("o")
        .long("only-matching")
        .help("Prints the matched bytes of each match")
        .overrides_with_all(&[
          "byte-offset",
          "files-with-matches",
          "files-without-matches",
        ])
    )
    .arg(
      Arg::with_name("byte-offset")
        .short("b")
        .long("byte-offset")
        .help("Prints the byte offset of each match")
        .overrides_with_all(&[
          "only-matching",
          "files-with-matches",
          "files-without-matches",
        ])
    )
    .arg(
      Arg::with_name("files-with-matches")
        .short("l")
        .long("files-with-matches")
        .help("Prints the name of the matched files (default output mode)")
        .overrides_with_all(&[
          "only-matching",
          "byte-offset",
          "files-without-matches",
        ])
    )
    .arg(
      Arg::with_name("files-without-matches")
        .short("L")
        .long("files-without-matches")
        .help("Prints the name of non-matched files")
        .overrides_with_all(&[
          "only-matching",
          "byte-offset",
          "files-with-matches",
        ])
    )
}


/// Build an `Args` from clap's `ArgMatches`.
/// The matches are supposed to be valid, therefore there is no error handling/reporting.
fn build_args(args: ArgMatches) -> Args {
  let pattern = args.value_of("pattern")
                    .expect("<pattern> not in ArgMatches") // pattern is required.
                    .to_owned();

  let files: Box<[PathBuf]> = args.values_of_os("files")
                                  .expect("<files> not in ArgMatches")
                                  .map(PathBuf::from)
                                  .collect();

  let flag = |f| args.is_present(f);

  let output_flags = (
    flag("only-matching"),
    flag("byte-offset"),
    flag("files-with-matches"),
    flag("files-without-matches")
  );

  let output = match output_flags {
    (true, _, _, _) => Output::Bytes,
    (_, true, _, _) => Output::Offset,
    (_, _, true, _) => Output::FileName,
    (_, _, _, true) => Output::FileName,
    (_, _, _, _)    => Default::default(),
  };

  Args {
    options: Options {
      inverse: flag("invert-match"),
      case_insensitive: flag("ignore-case"),
      trim_ending_newline: flag("trim-ending-newline"),
      non_matching: flag("files-without-matches"),
      print_filename: flag("with-filename") || !(flag("no-filename") || files.len() == 1),
      output
    },
    pattern,
    files
  }
}


/// Parse the arguments from `std::env::args_os`.
/// Returns the command to be executed, or the error message.
pub fn parse<
  T: Into<OsString> + Clone,
  A: IntoIterator<Item = T>
>(args: A) -> Result<Command, Error> {
  let app = build_app();

  match app.get_matches_from_safe(args) {
    Ok(arg_matches) => Ok(Command::Grep(build_args(arg_matches))),
    Err(e) => match e.kind {
      clap::ErrorKind::HelpDisplayed    => Ok(Command::Help(e.message)),
      clap::ErrorKind::VersionDisplayed => Ok(Command::Version(e.message)),
      _ => Err(Error { message: e.message })
    }
  }
}
