# const-css-minify

[<img alt="github" src="https://img.shields.io/badge/github-scpso%2Fconst--css--minify-7c72ff?logo=github">](https://github.com/scpso/const-css-minify)
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-const--css--minify-f46623?logo=rust">](https://crates.io/crates/const-css-minify)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-const--css--minify-ffc933?logo=docs.rs">](https://docs.rs/const-css-minify)
<img alt="status" src="https://img.shields.io/docsrs/const-css-minify/latest">

Include a minified css file as an inline const in your high-performance compiled web
application.

    use const_css_minify::minify;

    const CSS: &str = minify!("./path/to/style.css");

`const_css_minify` is not a good solution if your css changes out-of-step with your binary, as
you will not be able to change the css without recompiling your application.

#### `const_css_minify` ***will***:
* remove unneeded whitespace and linebreaks
* remove comments
* remove unneeded trailing semicolon in each declaration block
* opportunistically minify literal colors if and only if they can be expressed identically with
  a 3 character code (e.g. `#ffffff` will be substituted for `#fff` but `#fffffe` and
  `#ffffffff` will be left untouched)
* silently ignore any actual css syntax errors originating in your source file, and in so doing
  possibly elicit slightly different failure modes from renderers by altering the placement of
  whitespace around misplaced operators.

#### `const_css_minify` will ***not***:
* compress your css using `gz`, `br` or `deflate`
* change the semantic meaning of your semantically valid css
* make any substitutions other than identical literal colors
* do anything at all to alert you to invalid css - it's not truly parsing the css, just
  scanning for and removing characters it identifies as unnecessary.

This project is licensed under the terms of the MIT License.
