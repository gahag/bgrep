use clap::{self, App, Arg, ArgMatches};
use clap::{crate_authors, crate_version, crate_name, crate_description};


#[derive(Debug)]
pub enum Output {
  FileName,
  Bytes,
  Offset
}

impl Default for Output {
  fn default() -> Output { Output::FileName }
}


#[derive(Default, Debug)]
pub struct Options {
  pub inverse: bool,
  pub case_insensitive: bool,
  pub trim_ending_newline: bool,
  pub non_matching: bool, // Wheter to print non matching files. Only true when (-L).
  pub output: Output
}


#[derive(Default, Debug)]
pub struct Args {
  pub options: Options,
  pub pattern: String,
  pub files: Box<[String]>
}


#[derive(Debug)]
pub enum Command {
  Help(String),
  Version(String),
  Grep(Args)
}


#[derive(Debug)]
pub struct Error {
  pub message: String
}



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


fn build_args<'a>(args: ArgMatches<'a>) -> Args {
  let pattern = String::from(
    args.value_of("pattern")
        .expect("<pattern> not in ArgMatches") // pattern is required.
  );

  let files = match args.values_of("files") {
    None     => Box::new([String::from("-")]) as Box<[String]>, // Input from stdin.
    Some(fs) => fs.map(String::from).collect()
  };

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
      output
    },
    pattern,
    files
  }
}


pub fn parse() -> Result<Command, Error> {
  let app = build_app();

  match app.get_matches_safe() {
    Ok(arg_matches) => Ok(Command::Grep(build_args(arg_matches))),
    Err(e) => match e.kind {
      clap::ErrorKind::HelpDisplayed    => Ok(Command::Help(e.message)),
      clap::ErrorKind::VersionDisplayed => Ok(Command::Version(e.message)),
      _ => Err(Error { message: e.message })
    }
  }
}
