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
//!         color: rgb(0 255 100% / 0.8);
//!         margin: 10px 10px;
//!     }
//! "#);
//! assert_eq!(CSS, "input[type=\"radio\"]:checked,.button:hover{color:#0ffc;margin:10px 10px}");
//! ```
//!
//! Note also that the current version of `const_css_minify` does not support passing in a variable.
//! only the above two patterns of a path to an external file or a literal str will work.
//!
//! `const_css_minify` is not a good solution if your css changes out-of-step with your binary, as
//! you will not be able to change the css without recompiling your application.
//!
//! #### `const_css_minify` ***will:***
//! * remove unneeded whitespace and linebreaks
//! * remove comments
//! * remove unneeded trailing semicolon in each declaration block
//! * opportunistically minify colors specified either by literal hex values or by `rgb()` and
//!   `rgba()` functions (in either legacy syntax with commas or modern syntax without commas)
//!   without changing the color. e.g. `#ffffff` will be substituted with `#fff`, `rgb(254 253 252)`
//!   with `#fefdfc`, `rgba(20%, 40%, 60%, 0.8)` with `#369c`, etc. `const-css-minify` will not
//!   attempt to calculate nested/complicated/relative rgb expressions (which will be passed
//!   through unadulturated for the end user's browser to figure out for itself) but most
//!   simple/literal expressions will be resolved and minified.
//! * silently ignore css syntax errors originating in your source file*, and in so doing possibly
//!   elicit slightly different failure modes from renderers by altering the placement of
//!   whitespace around misplaced operators*
//!
//! #### `const_css_minify` will ***not:***
//! * compress your css using `gz`, `br` or `deflate`
//! * change the semantic meaning of your semantically valid css
//! * make any substitutions other than identical literal colors
//! * alert you to invalid css* - it's not truly parsing the css, just scanning for and removing
//!   characters it identifies as unnecessary
//!
//! note*: The current version of `const-css-minify` will emit compile-time warning messages for
//! some syntax errors, including unclosed quote strings and unclosed comments, which indicate an
//! error in the css (or a bug in `const-css-minify`), however these messages do not offer much
//! help to the user to locate the source of the error. Internally, these error states are
//! identifed and handled to avoid panicking due to indexing out-of-bounds, and so reporting the
//! error message at compile time is in a sense 'for free', but this is a non-core feature of the
//! library and may be removed in a future version if it turns out to do more harm than good. In
//! any case, `const-css-minify` generally assumes it is being fed valid css as input offers no
//! guarantees about warnings. `const-css-minify` should not be relied upon for linting of css.

use proc_macro::TokenStream;
use proc_macro::TokenTree::Literal;
use std::collections::HashMap;
use std::fmt;
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

    let mut minifier = Minifier::new();
    minifier.minify_string(&minified);
    minifier.emit_error_msgs();
    minified = minifier.get_output();

    // wrap in quotes, ready to emit as rust raw str token
    minified = "r####\"".to_string() + &minified + "\"####";

    TokenStream::from_str(&minified).unwrap()
}

struct ParseError {
    msg: String,
}

impl ParseError {
    pub fn from_msg(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// we do not attempt to decode all valid rgb func expressions, but we do attempt simple expressions
// that consist of purely literal numeric expressions.
const RGB_FUNC_DECODABLE: [u8; 15] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b' ', b',', b'%', b'.', b'/',
];

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
struct Minifier<'a> {
    input: Option<&'a [u8]>,
    output0: Vec<u8>,
    output1: Vec<u8>,
    // start and end indexes
    quotes0: HashMap<usize, usize>,
    errors: Vec<ParseError>,
}

impl<'a> Minifier<'a> {
    pub fn get_output(self) -> String {
        String::from_utf8(self.output1).unwrap()
    }

    pub fn new() -> Self {
        Self {
            input: None,
            output0: Vec::<u8>::with_capacity(0),
            output1: Vec::<u8>::with_capacity(0),
            quotes0: HashMap::<usize, usize>::new(),
            errors: Vec::<ParseError>::new(),
        }
    }

    pub fn minify_string(&mut self, input: &'a String) {
        self.input = Some(input.as_bytes());
        self.pass0();
        self.pass1();
    }

    fn add_error_msg(&mut self, msg: &str) {
        self.errors.push(ParseError::from_msg(msg));
    }

    fn emit_error_msgs(&self) {
        for error in &self.errors {
            eprintln!("WARN! const-css-minify parse error: {}", error);
        }
    }

    //collapse all whitespace sequences into single ' ', remove comments,
    //mark quotes in output stream
    fn pass0(&mut self) {
        let input = self.input.unwrap();
        let len = input.len();
        let mut output = Vec::<u8>::with_capacity(len);
        let mut read = 0;
        loop {
            match read {
                i if i == len => break,
                i if i > len => unreachable!(), // to catch errors of reasoning in indexing
                _ => (),
            }
            match input[read] {
                // trim excess whitespace, convert to space
                w if w.is_ascii_whitespace() => {
                    // if the last element was a comment that was entirely ignored, and if the
                    // comment was preceeded by whitespace, we might end up with two consecutive
                    // whitespaces, which violates the promise of this method. Thus we explicitly
                    // check and remove it if present.
                    if let Some(last) = output.pop() {
                        if last != b' ' {
                            output.push(last);
                        }
                    }
                    read += 1;
                    while read < len && input[read].is_ascii_whitespace() {
                        read += 1;
                    }
                    // don't add whitespace to head or tail
                    if !output.is_empty() && read < len {
                        output.push(b' ');
                    }
                }
                // css comments
                b'/' if len > read + 1 && input[read + 1] == b'*' => {
                    let mut found_end = false;
                    // move read index to first char after '*' in the matched pattern, or possibly
                    // past the end of input if '/*' are the last two chars.
                    read += 2;
                    // below we are comparing against a '*' at read - 1, and we explicitly want to
                    // avoid opening and closing a comment on '/*/' - a correct comment consists of
                    // '/**/ at a minimum. Therefore we must increment read once more, but we only
                    // want to do this if we aren't already beyond the end of input
                    if read < len {
                        read += 1;
                    }
                    while read < len {
                        let s = &input[read - 1..=read];
                        read += 1;
                        if s == [b'*', b'/'] {
                            found_end = true;
                            break;
                        }
                    }
                    if !found_end {
                        self.add_error_msg("reached end of input while inside comment");
                    }
                }
                // quotes
                q @ (b'"' | b'\'') => {
                    let start = output.len();
                    output.push(input[read]);
                    read += 1;
                    let mut found_end = false;
                    while read < len {
                        let b = input[read];
                        output.push(b);
                        read += 1;
                        if b == q {
                            found_end = true;
                            break;
                        }
                    }
                    if !found_end {
                        self.add_error_msg("reached end of input while inside quote string");
                    }
                    let end = output.len() - 1;
                    self.quotes0.insert(start, end);
                }
                _ => {
                    output.push(input[read]);
                    read += 1;
                }
            }
        }
        self.output0 = output;
    }

    fn pass1(&mut self) {
        let input = &self.output0;
        let len = input.len();
        let mut output = Vec::<u8>::with_capacity(len);
        let mut read = 0;
        let mut peek;
        let mut backreference = None;
        loop {
            match read {
                i if i == len => break,
                i if i > len => unreachable!(), // to catch errors of reasoning in indexing
                _ => (),
            }
            match input[read] {
                // copy quotes verbatim
                b'\'' | b'"' => {
                    let end = self.quotes0.get(&read).unwrap();
                    while read <= *end {
                        output.push(input[read]);
                        read += 1
                    }
                }
                // enter declaration block
                b'{' => {
                    backreference = None;
                    if let Some(last) = output.pop() {
                        if last != b' ' {
                            output.push(last);
                        }
                    }
                    output.push(input[read]);
                    read += 1;
                    // drop trailing space
                    if read < len && input[read] == b' ' {
                        read += 1;
                    }
                }
                // exit declaration block
                b'}' => {
                    if let Some(br) = backreference {
                        output.remove(br);
                    }
                    backreference = None;
                    if let Some(last) = output.pop() {
                        if last != b' ' {
                            output.push(last);
                        }
                    }
                    // drop final semicolon in declaration block
                    if let Some(last) = output.pop() {
                        if last != b';' {
                            output.push(last);
                        }
                    }
                    output.push(input[read]);
                    read += 1;
                    // drop trailing space
                    if read < len && input[read] == b' ' {
                        read += 1;
                    }
                }
                // value assignement OR pseudo class/element
                b':' => {
                    backreference = None;
                    // pseudo element
                    if len > read + 1 && input[read + 1] == b':' {
                        output.push(b':');
                        output.push(b':');
                        read += 2;
                    } else {
                        if let Some(last) = output.pop() {
                            // mark backreference for possible future removal
                            if last == b' ' {
                                backreference = Some(output.len());
                            }
                            output.push(last);
                        }
                        output.push(input[read]);
                        read += 1;
                        // drop trailing space
                        if read < len && input[read] == b' ' {
                            read += 1;
                        }
                    }
                }
                // comma separator
                b',' => {
                    // drop spaces preceeding commas
                    if let Some(last) = output.pop() {
                        if last != b' ' {
                            output.push(last);
                        }
                    }
                    output.push(input[read]);
                    read += 1;
                    // drop trailing space
                    if read < len && input[read] == b' ' {
                        read += 1;
                    }
                }
                // semicolon separator
                b';' => {
                    if let Some(br) = backreference {
                        output.remove(br);
                    }
                    backreference = None;
                    // drop leading space
                    if let Some(last) = output.pop() {
                        if last != b' ' {
                            output.push(last);
                        }
                    }
                    output.push(input[read]);
                    read += 1;
                    // drop trailing space
                    if read < len && input[read] == b' ' {
                        read += 1;
                    }
                }

                // possible hex color
                b'#' if len > read + 3 => {
                    peek = read + 1;
                    while len > peek && input[peek].is_ascii_hexdigit() {
                        peek += 1;
                    }
                    if let Ok(mut hex_color) = try_minify_hex_color(&input[read..peek]) {
                        output.append(&mut hex_color);
                        read = peek;
                    } else {
                        output.push(input[read]);
                        read += 1;
                    }
                }
                // possible rgb func
                b'r' if len > read + 9
                    && (input[read + 1..=read + 3] == [b'g', b'b', b'(']
                        || input[read + 1..=read + 4] == [b'g', b'b', b'a', b'(']) =>
                {
                    peek = read + 4;
                    if input[peek] == b'(' {
                        peek += 1;
                    }
                    while len > peek
                        && input[peek] != b')'
                        && RGB_FUNC_DECODABLE.contains(&input[peek])
                    {
                        peek += 1
                    }
                    if input[peek] == b')' {
                        if let Ok(mut hex_color) = try_decode_rgb_func(&input[read..=peek]) {
                            hex_color = try_minify_hex_color(&hex_color).unwrap();
                            output.append(&mut hex_color);
                            read = peek + 1;
                            continue;
                        }
                    }
                    output.push(input[read]);
                    read += 1;
                }
                // all else copy verbatim
                _ => {
                    output.push(input[read]);
                    read += 1;
                }
            }
        }
        self.output0.clear();
        self.output0.shrink_to_fit();
        self.output1 = output;
    }
}

/*
 * requires input to start with "rgb(" or "rgba(" and end with ")"
 */
fn try_decode_rgb_func(input: &[u8]) -> Result<Vec<u8>, ()> {
    let mut v = vec![b'#'];
    let mut read = 3;
    if input[read] == b'a' {
        read += 1;
    }
    if input[read] != b'(' {
        return Err(());
    }
    read += 1;
    let mut rgba_d = [
        String::with_capacity(10),
        String::with_capacity(10),
        String::with_capacity(10),
        String::with_capacity(10),
    ];
    let mut percents = [false, false, false, false];
    let mut i = 0;
    while input[read] != b')' {
        match input[read] {
            x if !RGB_FUNC_DECODABLE.contains(&x) => return Err(()),
            d if d.is_ascii_digit() || d == b'.' => rgba_d[i].push(char::from(d)),
            b'%' => percents[i] = true,
            b' ' | b',' | b'/' => {
                i += 1;
                while [b' ', b',', b'/'].contains(&input[read + 1]) {
                    read += 1;
                }
            }
            _ => unreachable!(), // did we add chars to RGB_FUNC_DECODABLE and not match here?
        }
        read += 1;
    }
    // check we got required input for r, g, b
    for i in 0..=2 {
        if rgba_d[i].is_empty() {
            return Err(());
        }
        let byte: u8 = if percents[i] {
            let decimal = f32::from_str(&rgba_d[i]).or(Err(()))?; // 👈 #unexpectedlisp
            let integer = (decimal * 255_f32 / 100_f32).round();
            if integer < u8::MIN.into() || integer > u8::MAX.into() {
                return Err(());
            }
            unsafe { integer.to_int_unchecked() }
        } else {
            u8::from_str(&rgba_d[i]).or(Err(()))?
        };
        //format as hexadecimal
        let hex = format!("{:04x}", byte).into_bytes();
        //igore leading '0x' get only the actual hexadecimal digits
        v.push(hex[2]);
        v.push(hex[3]);
    }
    // alpha channel
    if !rgba_d[3].is_empty() && !["1", "1.0", "100"].contains(&rgba_d[3].as_str()) {
        let decimal = f32::from_str(&rgba_d[3]).or(Err(()))?;
        let integer = if percents[3] {
            (decimal * 255_f32 / 100_f32).round()
        } else {
            (decimal * 255_f32).round()
        };
        if integer < u8::MIN.into() || integer > u8::MAX.into() {
            return Err(());
        }
        let byte: u8 = unsafe { integer.to_int_unchecked() };

        //format as hexadecimal
        let hex = format!("{:04x}", byte).into_bytes();
        //igore leading '0x' get only the actual hexadecimal digits
        v.push(hex[2]);
        v.push(hex[3]);
    }
    Ok(v)
}

fn try_minify_hex_color(input: &[u8]) -> Result<Vec<u8>, ()> {
    let len = input.len();
    if ![4, 5, 7, 9].contains(&len) || input[0] != b'#' {
        return Err(());
    }
    let mut v = vec![b'#'];
    for byte in &input[1..] {
        if !byte.is_ascii_hexdigit() {
            return Err(());
        }
        v.push(*byte);
    }
    if len == 9 && v[1] == v[2] && v[3] == v[4] && v[5] == v[6] && v[7] == v[8] {
        v.remove(8);
        v.remove(6);
        v.remove(4);
        v.remove(2);
    }
    if len == 7 && v[1] == v[2] && v[3] == v[4] && v[5] == v[6] {
        v.remove(6);
        v.remove(4);
        v.remove(2);
    }
    Ok(v)
}
