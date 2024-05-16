# const-css-minify

[<img alt="github" src="https://img.shields.io/badge/github-scpso%2Fconst--css--minify-7c72ff?logo=github">](https://github.com/scpso/const-css-minify)
[<img alt="crates.io" src="https://img.shields.io/crates/v/const-css-minify.svg?logo=rust">](https://crates.io/crates/const-css-minify)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/const-css-minify/latest?logo=docs.rs">](https://docs.rs/const-css-minify)

Include a minified css file as an inline const in your high-performance compiled web
application.

    use const_css_minify::minify;

    const CSS: &str = minify!("./path/to/style.css");

`const_css_minify` is not a good solution if your css changes out-of-step with your binary, as
you will not be able to change the css without recompiling your application.

#### `const_css_minify` ***will:***
* remove unneeded whitespace and linebreaks
* remove comments
* remove unneeded trailing semicolon in each declaration block
* opportunistically minify colors specified either by literal hex values or by `rgb()`,
  `rgba()`, `hsl()` and `hsla()` functions (in either legacy syntax with commas or modern
  syntax without commas) without changing the color. e.g. `#ffffff` will be substituted with
  `#fff`, `hsl(180 50 50)` with `#40bfbf`, `rgba(20%, 40%, 60%, 0.8)` with `#369c`, etc.
  `const-css-minify` will not attempt to calculate nested/complicated/relative rgb expressions
  (which will be passed through unadulturated for the end user's browser to figure out for
  itself) but many simple/literal expressions will be resolved and minified.
* silently ignore css syntax errors originating in your source file*, and in so doing possibly
  elicit slightly different failure modes from renderers by altering the placement of
  whitespace around misplaced operators

#### `const_css_minify` will ***not:***
* compress your css using `gz`, `br` or `deflate`
* change the semantic meaning of your semantically valid css
* make any substitutions other than identical literal colors
* alert you to invalid css* - it's not truly parsing the css, just scanning for and removing
  characters it identifies as unnecessary

`const_css_minify` is a lightweight solution - the current version of `const_css_minify` has
zero dependencies outside rust's built-in std and proc_macro libraries.

This project is licensed under the terms of the MIT License.
