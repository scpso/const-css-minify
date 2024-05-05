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

    #[test]
    fn hexcoded_6chars() {
        assert_eq!(minify!("#{color:#aabbcc}"), "#{color:#abc}");
        assert_eq!(minify!("#{color:#DDEEFF}"), "#{color:#def}");
    }

    #[test]
    fn hexcoded_8chars() {
        assert_eq!(minify!("#{color:#aabbccdd}"), "#{color:#aabbccdd}");
    }

    #[test]
    fn rgbfunc_long() {
        assert_eq!(minify!("#{color:rgb(255,255,254)}"), "#{color:#fffffe}");
    }

    #[test]
    fn rgbfunc_short() {
        assert_eq!(minify!("#{color:rgb(0, 0, 0)}"), "#{color:#000}");
    }

    #[test]
    fn rgbfunc_percent() {
        assert_eq!(minify!("#{color:rgb(0%, 0%, 0%)}"), "#{color:#000}");
        assert_eq!(minify!("#{color:rgb(1%, 2%, 3%)}"), "#{color:#020507}");
        assert_eq!(minify!("#{color:rgb(4%, 5%, 6%)}"), "#{color:#0a0c0f}");
        assert_eq!(minify!("#{color:rgb(7%, 8%, 9%)}"), "#{color:#111416}");
        assert_eq!(minify!("#{color:rgb(20%, 20%, 20%)}"), "#{color:#333}");
        assert_eq!(minify!("#{color:rgb(40%, 40%, 40%)}"), "#{color:#666}");
        assert_eq!(minify!("#{color:rgb(50%, 50%, 50%)}"), "#{color:#7f7f7f}");
        assert_eq!(minify!("#{color:rgb(60%, 60%, 60%)}"), "#{color:#999}");
        assert_eq!(minify!("#{color:rgb(80%, 80%, 80%)}"), "#{color:#ccc}");
        assert_eq!(minify!("#{color:rgb(100%, 100%, 100%)}"), "#{color:#fff}");
    }
}
