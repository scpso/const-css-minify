//! [<img alt="github" src="https://img.shields.io/badge/github-scpso%2Fconst--css--minify-7c72ff?logo=github">](https://github.com/scpso/const-css-minify)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/const-css-minify.svg?logo=rust">](https://crates.io/crates/const-css-minify)
//! [<img alt="docs.rs" src="https://img.shields.io/docsrs/const-css-minify/latest?logo=docs.rs">](https://docs.rs/const-css-minify)
//!
//! Include a minified css file as an inline const in your high-performance compiled web
//! application.
//!
//! You can call it with a path to your source css file, just like you might use the built-in
//! macro `include_str!()`:
//!
//! ```rust
//! use const_css_minify::minify;
//!
//! // this is probably the pattern you want to use
//! const CSS: &str = minify!("./path/to/style.css");
//! ```
//!
//! <div class="warning">
//!
//! ***IMPORTANT!*** the current version of `const_css_minify` resolves paths relative to the crate
//! root (i.e. the directory where your `Cargo.toml` is). This behaviour is ***DIFFERENT*** from the
//! rust built-in macros like `include_str!()` which use a path relative to the source file from
//! which it's invoked. Consider the current behaviour unstable and likely to change  - our
//! preference would be to match the established convention, but implementing this change is
//! dependant on the stabilisation of a source path api in `proc_macro` as per
//! <https://github.com/rust-lang/rust/issues/54725>
//!
//! </div>
//!
//! It's also possible to include a raw string with your css directly in your rust source:
//! ```rust
//! use const_css_minify::minify;
//!
//! const CSS: &str = minify!(r#"
//!     input[type="radio"]:checked, .button:hover {
//!         color: rgb(100%, 100%, 100%);
//!         margin: 10px 10px;
//!     }
//! "#);
//! assert_eq!(CSS, "input[type=\"radio\"]:checked,.button:hover{color:#fff;margin:10px 10px}");
//! ```
//!
//! Note also that the current version of `const_css_minify` does not support passing in a variable.
//! only the above two patterns of a path to an external file or a literal str will work.
//!
//! `const_css_minify` is not a good solution if your css changes out-of-step with your binary, as
//! you will not be able to change the css without recompiling your application.
//!
//! #### `const_css_minify` ***will***:
//! * remove unneeded whitespace and linebreaks
//! * remove comments
//! * remove unneeded trailing semicolon in each declaration block
//! * opportunistically minify literal hex colors if and only if they can be expressed identically
//!   with a 3 character code (e.g. `#ffffff` will be substituted with `#fff` but `#fffffe` and
//!   `#ffffffff` will be left untouched)
//! * minify colors specified by `rgb` function (e.g. `rgb(255, 255, 254)` will be substituted with
//!   `#fffffe`, and `rgb(255, 255, 255)` with `#fff`)
//! * silently ignore any actual css syntax errors originating in your source file, and in so doing
//!   possibly elicit slightly different failure modes from renderers by altering the placement of
//!   whitespace around misplaced operators.
//!
//! #### `const_css_minify` will ***not***:
//! * compress your css using `gz`, `br` or `deflate`
//! * change the semantic meaning of your semantically valid css
//! * make any substitutions other than identical literal colors
//! * do anything at all to alert you to invalid css - it's not truly parsing the css, just
//!   scanning for and removing characters it identifies as unnecessary.

use proc_macro::TokenStream;
use proc_macro::TokenTree::Literal;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Produce a minified css file as an inline const
#[proc_macro]
pub fn minify(input: TokenStream) -> TokenStream {
    let token_trees: Vec<_> = input.into_iter().collect();
    if token_trees.len() != 1 {
        panic!("const_css_minify requires a single str as input");
    }
    let Literal(literal) = token_trees.first().unwrap() else {
        panic!("const_css_minify requires a literal str as input");
    };
    let mut literal = literal.to_string();

    // not a raw string, so we must de-escape special chars
    // this is not comprehensive but is anyone ever going to even notice?
    // what weird and strange things might they even be trying to achieve?
    if let Some(c) = literal.get(0..=0) {
        if c != "r" {
            literal = literal
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .replace("\\\\", "\\")
        }
    }

    // trim leading and trailing ".." or r#".."# from string literal
    let start = &literal.find('\"').unwrap() + 1;
    let end = &literal.rfind('\"').unwrap() - 1;
    //bail if literal is empty
    if start > end {
        return TokenStream::from_str(&literal).unwrap();
    }
    literal = literal[start..=end].to_string();

    // check if we're dealing with path or literal
    let mut minified = fs::read_to_string(Path::new(&literal)).unwrap_or(literal);

    minified = parse(minified);

    // wrap in quotes, ready to emit as rust raw str token
    minified = "r####\"".to_string() + &minified + "\"####";

    TokenStream::from_str(&minified).unwrap()
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Before,
    During,
}

fn parse(input: String) -> String {
    /*
     * css is relatively simple but there are a few gotchas. Nested classes basically means any
     * property can be a selector, so we can't generically distinguish between the two without
     * a lookup to known legal names, and also the fact that pseudo classes and elements are
     * denoted with ':' which is also the value assignment operator means we need to scan ahead to
     * decide if a particular ':' on the input is part of a selector and requires leading
     * whitespace to be preserved, or if it's the assignment operator and doesn't require leading
     * whitespace. To avoid re-implementing comment and quote handling while scanning forward, we
     * instead mark the index as a backreference and remove it later if we can. This also has the
     * conseqence that we also cannot generically identify if we are currently parsing a property
     * or a value without a lookup to known legal names, which as far as I know shouldn't cause
     * problems for handling correct css but eliminates some avenues for error tolerance. But
     * intelligent handling of incorrect css is beyond this scope of this crate so this is
     * acceptable.
     */
    let input = input.as_bytes();
    let mut minified = Vec::<u8>::with_capacity(input.len());
    let mut read = 0;
    let mut peek;
    let mut backreference = None;
    let mut byte;
    let mut state = State::Before;
    loop {
        if read == input.len() {
            return String::from_utf8(minified).unwrap();
        }
        byte = input[read];
        match (byte, state) {
            // ignore leading whitespace
            (b' ' | b'\t' | b'\r' | b'\n', State::Before) => read += 1,
            // trim excess whitespace, convert to space
            (b' ' | b'\t' | b'\r' | b'\n', State::During) => {
                peek = read + 1;
                while [b' ', b'\t', b'\r', b'\n'].contains(&input[peek]) {
                    peek += 1;
                }
                minified.push(b' ');
                read = peek;
            }
            // identify and ignore comments
            (b'/', _) if input[read + 1] == b'*' => {
                peek = read + 2;
                while !(input[peek] == b'*' && input[peek + 1] == b'/') {
                    peek += 1;
                }
                read = peek + 2;
            }
            // identify and consume double quote strings
            (b'"', _) => {
                peek = read + 1;
                while input[peek] != b'"' {
                    peek += 1;
                }
                minified.extend_from_slice(&input[read..=peek]);
                read = peek + 1;
                state = State::During;
            }
            // identify and consume single quote strings
            (b'\'', _) => {
                peek = read + 1;
                while input[peek] != b'\'' {
                    peek += 1;
                }
                minified.extend_from_slice(&input[read..=peek]);
                read = peek + 1;
                state = State::During;
            }
            // enter declaration block
            (b'{', _) => {
                backreference = None;
                if let Some(last) = minified.pop() {
                    if last != b' ' {
                        minified.push(last);
                    }
                }
                minified.push(byte);
                read += 1;
                state = State::Before;
            }
            // exit declaration block
            (b'}', _) => {
                if let Some(j) = backreference {
                    minified.remove(j);
                }
                backreference = None;
                if let Some(last) = minified.pop() {
                    if last != b';' && last != b' ' {
                        minified.push(last);
                    }
                }
                minified.push(byte);
                read += 1;
                state = State::Before;
            }
            // comma separator
            (b',', State::During) => {
                if let Some(last) = minified.pop() {
                    if last != b' ' {
                        minified.push(last);
                    }
                }
                minified.push(byte);
                peek = read + 1;
                while [b' ', b'\t', b'\r', b'\n'].contains(&input[peek]) {
                    peek += 1;
                }
                read = peek;
            }
            // value assignement OR pseudo class/element
            (b':', _) => {
                backreference = None;
                // pseudo element
                if input[read + 1] == b':' {
                    minified.push(b':');
                    minified.push(b':');
                    read += 2;
                } else {
                    if let Some(last) = minified.pop() {
                        // mark backreference for possible future removal
                        if last == b' ' {
                            backreference = Some(minified.len());
                        }
                        minified.push(last);
                    }
                    minified.push(byte);
                    peek = read + 1;
                    while [b' ', b'\t', b'\r', b'\n'].contains(&input[peek]) {
                        peek += 1;
                    }
                    read = peek;
                    state = State::During;
                }
            }
            // end of value
            (b';', _) => {
                if let Some(j) = backreference {
                    minified.remove(j);
                }
                backreference = None;
                minified.push(byte);
                read += 1;
                state = State::Before;
            }
            // possible hex color
            (b'#', State::During) if input.len() > read + 6 => {
                minified.push(byte);
                let colors = [
                    [input[read + 1], input[read + 2]],
                    [input[read + 3], input[read + 4]],
                    [input[read + 5], input[read + 6]],
                ];
                // avoid 8 hex char codes with alpha
                if !input[read + 7].is_ascii_hexdigit()
                    && colors[0][0].is_ascii_hexdigit()
                    && colors[0][0] == colors[0][1]
                    && colors[1][0].is_ascii_hexdigit()
                    && colors[1][0] == colors[1][1]
                    && colors[2][0].is_ascii_hexdigit()
                    && colors[2][0] == colors[2][1]
                {
                    minified.extend_from_slice(&[
                        colors[0][0].to_ascii_lowercase(),
                        colors[1][0].to_ascii_lowercase(),
                        colors[2][0].to_ascii_lowercase(),
                    ]);
                    read += 7;
                } else {
                    read += 1;
                }
            }
            // possible rgb color
            (b'r', State::During)
                if input.len() > read + 8 && input[read + 1..=read + 3] == [b'g', b'b', b'('] =>
            {
                let mut rgb_d = [
                    String::with_capacity(3),
                    String::with_capacity(3),
                    String::with_capacity(3),
                ];
                let mut rgb_u: [u8; 3] = [0; 3];
                let mut rgb_h: [[u8; 2]; 3] = [[0, 0]; 3];
                let mut shorts = [false, false, false];
                let mut percents = [false, false, false];
                let mut bail = false;
                let mut i = 0;
                peek = read + 4;
                while input[peek] != b')' {
                    //bail if we hit a comment or too many commas
                    if input[peek] == b'/' || i == 3 {
                        bail = true;
                        break;
                    }
                    if input[peek].is_ascii_digit() {
                        rgb_d[i].push(char::from(input[peek]));
                    } else if input[peek] == b'%' {
                        percents[i] = true;
                    } else if input[peek] == b',' {
                        i += 1;
                    }
                    peek += 1;
                }
                //check we got expected input
                for j in 0..=2 {
                    if rgb_d[j].is_empty() {
                        bail = true;
                    } else if percents[j] {
                        let dec = u8::from_str(&rgb_d[j]).unwrap_or_else(|_| {
                            bail = true;
                            0
                        });
                        if !bail {
                            rgb_u[j] = u8::try_from(u32::from(dec) * 255 / 100).unwrap();
                        }
                    } else {
                        rgb_u[j] = u8::from_str(&rgb_d[j]).unwrap_or_else(|_| {
                            bail = true;
                            0
                        });
                    }
                }
                if bail || i != 2 {
                    minified.push(input[read]);
                    read += 1;
                } else {
                    for j in 0..=2 {
                        if rgb_u[j] % 17 == 0 {
                            shorts[j] = true;
                        }
                        //format as hexadecimal
                        let bytes = format!("{:04x}", rgb_u[j]).into_bytes();
                        //igore leading '0x' get only the actual hexadecimal digits
                        rgb_h[j] = [bytes[2], bytes[3]];
                    }
                    minified.push(b'#');
                    if shorts == [true, true, true] {
                        for hex in rgb_h {
                            minified.push(hex[0]);
                        }
                    } else {
                        for hex in rgb_h {
                            minified.extend_from_slice(&hex);
                        }
                    }
                    read = peek + 1;
                }
            }
            _ => {
                minified.push(byte);
                read += 1;
                state = State::During;
            }
        }
    }
}
