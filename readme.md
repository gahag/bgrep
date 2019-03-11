# Bgrep - A binary grep written in Rust.

Bgrep is a grep spin that aims to support binary patterns and files. The key difference
from its cousins is that it won't do line-wise matching. Therefore, you can match
any byte pattern, including those that would span multiple lines.


## Usage

```
Usage: target/debug/bgrep [OPTIONS] PATTERN [FILES...]

Options:
    -h, --help          print usage message
    -l, --files-with-matches 
                        print the name of the matched files
    -o, --only-matching 
                        print the matched bytes of each match
    -p, --position      print the byte offset of each match
    -v, --invert-match  inverse matching
```

Bgrep uses Rust's [regex crate](https://crates.io/crates/regex) as engine. The regex
syntax documentation can be found [here](https://docs.rs/regex/1.1.2/regex/#syntax). Bgrep
disables unicode matching by default, but it can be enabled with the `u` regex pattern
flag.


## License

Copyright &copy; 2019 gahag.  
All rights reserved.

This software may be modified and distributed under the terms
of the BSD 3 Clause license. See the [LICENSE](LICENSE) file for details.
