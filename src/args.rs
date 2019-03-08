use std::env;
use std::io;

use getopts::Options as OptParser;


pub enum Output {
  Filename,
  Bytes,
  Position
}

impl Default for Output {
  fn default() -> Output {
    Output::Filename
  }
}


#[derive(Default)]
pub struct Options {
  pub help: Option<String>,
  pub inverse: bool,
  pub output: Output
}


#[derive(Default)]
pub struct Args {
  pub options: Options,
  pub pattern: String,
  pub files: Box<[String]>
}



pub fn parse() -> Result<Args, io::ErrorKind> {
  let mut optparser = OptParser::new();
  optparser.optflag("h", "help", "print usage message")
           .optflag("l", "files-with-matches", "print the name of the matched files")
           .optflag("o", "only-matching", "print the matched bytes of each match")
           .optflag("p", "position", "print the byte offset of each match")
           .optflag("v", "invert-match", "inverse matching");

  let usage = |program: &str| {
    optparser.usage(&format!("Usage: {} [OPTIONS] PATTERN [FILES...]", program))
  };

  let args_error = |program: &str| {
    eprint!("{}", usage(program));
    io::ErrorKind::InvalidInput
  };


  let mut args = env::args();

  let program = args.next().ok_or_else(|| args_error("<program name>"))?;

  let opts = optparser.parse(args).map_err(
    |e| {
      eprintln!("Error: {:?}", e);
      args_error(&program)
    }
  )?;

  if opts.opt_present("h") {
    return Ok(
      Args {
        options: Options {
          help: Some(usage(&program)),
          .. Default::default()
        },
        .. Default::default()
      }
    );
  };

  let grep_l = opts.opt_present("l");
  let grep_o = opts.opt_present("o");
  let grep_p = opts.opt_present("p");

  let error_exclusive = |x: &str, y: &str| {
    eprintln!("Error: '{}' and '{}' are exclusive.", x, y);
    Err(io::ErrorKind::InvalidInput)
  };

  let output = match (grep_l, grep_o, grep_p) {
    (true,  true,  _    ) => error_exclusive("-l", "-o"),
    (true,  _,     true ) => error_exclusive("-l", "-p"),
    (_,     true,  true ) => error_exclusive("-o", "-p"),
    (false, false, false) => Ok(Default::default()),
    (true,  _,     _    ) => Ok(Output::Filename),
    (_,     true,  _    ) => Ok(Output::Bytes),
    (_,     _,     true ) => Ok(Output::Position),
  }?;

  let options = Options {
    help: None,
    output,
    inverse: opts.opt_present("v")
  };

  let mut free = opts.free;

  if free.is_empty() {
    return Err(args_error(&program));
  }

  let pattern = free.remove(0);

  let files = if free.is_empty() {
    Box::new([String::from("-")]) // No input files -> input from stdin.
  }
  else {
    free.into_boxed_slice()
  };

  Ok(
    Args {
      options,
      pattern,
      files
    }
  )
}
