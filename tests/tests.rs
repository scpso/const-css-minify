#[cfg(test)]
mod tests {
    use const_css_minify::minify;

    /*
     * does not compile (with help message), which is the desired behaviour.
     * can test this manually by uncommenting.
     * not sure how to assert that it shouldn't be able to compile, and given
     * the behaviour is correct I don't think it's worth my time to try and
     * figure out how to test this properly.
    fn no_input() {
        const RESULT: &str = minify!();
    }
    */

    /*
     * does not compile (with help message), which is desired behaviour
    fn with_variable() {
        let css = "";
        const RESULT: &str = minify!(css);
    }
    */

    /*
     * ensure we can actually use this macro in const context, which after all
     * is the whole point
     */
    #[test]
    fn is_const() {
        const RESULT: &str = minify!("#{color:#fff}");
        assert_eq!(RESULT, "#{color:#fff}");
    }

    /*
     * ensure we can actually load an external css file
     */
    #[test]
    fn finds_css_file() {
        assert_eq!(minify!("./tests/test.css"), "#{color:#fff}");
    }

    #[test]
    fn empty_str() {
        assert_eq!(minify!(""), "",);
    }

    #[test]
    fn already_minified() {
        assert_eq!(minify!("#{color:#fff}"), "#{color:#fff}",);
    }

    #[test]
    fn unneeded_whitespace() {
        assert_eq!(minify!("# {color:#fff}"), "#{color:#fff}",);
    }

    #[test]
    fn required_whitespace() {
        assert_eq!(minify!("#{margin:1px 1px}"), "#{margin:1px 1px}",)
    }

    #[test]
    fn trailing_semicolon() {
        assert_eq!(minify!("#{margin:1px;}"), "#{margin:1px}",);
    }

    #[test]
    fn comments() {
        assert_eq!(minify!("#{margin:1px /*1px*/}"), "#{margin:1px}",);
    }

    #[test]
    fn pseudo_selectors() {
        assert_eq!(minify!("div :hover ::after{}"), "div :hover ::after{}");
    }

    #[test]
    fn nested_classes() {
        assert_eq!(minify!("div { span {margin:1px}}"), "div{span{margin:1px}}");
    }
}
