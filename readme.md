# Bgrep - A binary grep written in Rust.

Bgrep is a grep spin that aims to support binary patterns and files. The key difference
from its cousins is that it won't do line-wise matching. Therefore, you can match
any byte pattern, including those that would span multiple lines.


## Usage

```
bgrep [FLAGS] <pattern> [files]...

FLAGS:
    -b, --byte-offset              Prints the byte offset of each match
    -l, --files-with-matches       Prints the name of the matched files (default output mode)
    -L, --files-without-matches    Prints the name of non-matched files
        --help                     Prints help information
    -i, --ignore-case              Case insensitive matching for ASCII alphabetic characters
    -v, --invert-match             Invert the sense of matching, to select non matching slices
    -h, --no-filename              Suppress the file names on output (default when there is a single file).
    -o, --only-matching            Prints the matched bytes of each match
    -n, --trim-ending-newline      If the file ends with a newline, disconsider the last byte
    -V, --version                  Prints version information
    -H, --with-filename            Print the file name for each match (default when there are multiple files).
```

Bgrep uses Rust's [regex crate](https://crates.io/crates/regex) as engine. The regex
syntax documentation can be found [here](https://docs.rs/regex/1.1.2/regex/#syntax). Bgrep
disables unicode matching by default, but it can be enabled with the `u` regex pattern
flag.


## Examples

`file.bin`:
```
Ix |  0 1  2 3  4 5  6 7  8 9  A B  C D  E F | ASCII characters
---+-----------------------------------------+-----------------
00 | 0001 0203 0405 0607 0809 0A0B 0C0D 0E0F | ................
10 | 0000 0000 0000 0000 0000 0000 0000 0000 | ................
20 | 2021 2223 2425 2627 2829 2A2B 2C2D 2E2F |  !"#$%&'()*+,-./
30 | 0000 0000 0000 0000 0000 0000 0000 0000 | ................
40 | 4041 4243 4445 4647 4849 4A4B 4C4D 4E0A | @ABCDEFGHIJKLMN.
```

Get the offsets of a given pattern:
```
$ bgrep -b '\x00\x20|\x00\x40' file.bin
0x1f
0x3f
```

Get the matching bytes:
```
$ bgrep -o '\x40.+\x4d' file.bin
@ABCDEFGHIJKLM
```

List non-matching files: (`tac -rs .` reverses the file)
```
$ tac -rs . file.bin | bgrep -L '\x4C\x4D' file.bin -
<stdin>
```

Get the offset of inverse matches:
```
$ bgrep -bv '\x00+' file.bin
0x1
0x20
0x40
```


## License

Copyright &copy; 2019 gahag.  
All rights reserved.

This software may be modified and distributed under the terms
of the BSD 3 Clause license. See the [LICENSE](LICENSE) file for details.
