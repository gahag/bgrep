use std::env;
use std::io;


use getopts::Options;



pub enum GrepOutput {
  Filename,
  Bytes,
  Position
}

impl Default for GrepOutput {
  fn default() -> GrepOutput {
    GrepOutput::Filename
  }
}


#[derive(Default)]
pub struct Args {
  pub help: Option<String>,
  pub output: GrepOutput,
  pub pattern: String,
  pub files: Vec<String>
}



pub fn parse() -> Result<Args, io::ErrorKind> {
  let mut optparser = Options::new();
  optparser.optflag("h", "help", "print usage message")
           .optflag("l", "files-with-matches", "print the name of the matched files")
           .optflag("o", "only-matching", "print the matched bytes of each match")
           .optflag("p", "position", "print the byte offset of each match");

  let usage = |program: &str| {
    optparser.usage(&format!("Usage: {} [OPTIONS] PATTERN [FILES...]", program))
  };

  let args_error = |program: &str| {
    eprint!("{}", usage(program));
    io::ErrorKind::InvalidInput
  };


  let mut args: Vec<String> = env::args().collect();

  if args.is_empty() {
    return Err(args_error("<program name>"));
  }

  let program = args.remove(0);

  let opts = optparser.parse(args).map_err(
    |e| {
      eprintln!("Error: {:?}", e);
      args_error(&program)
    }
  )?;

  if opts.opt_present("h") {
    return Ok(
      Args {
        help: Some(usage(&program)),
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
    (true,  _,     _    ) => Ok(GrepOutput::Filename),
    (_,     true,  _    ) => Ok(GrepOutput::Bytes),
    (_,     _,     true ) => Ok(GrepOutput::Position),
  }?;

  let mut free = opts.free;

  if free.is_empty() {
    return Err(args_error(&program));
  }

  Ok(
    Args {
      help: None,
      output,
      pattern: free.remove(0),
      files: free
    }
  )
}
