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
    fn quotes() {
        //raw str with double quotes
        assert_eq!(
            minify!(r#"#{font-family:"Times New Roman", "Courier New"}"#),
            r#"#{font-family:"Times New Roman","Courier New"}"#,
        );
        //str with escaped double quotes
        assert_eq!(
            minify!("#{font-family:\"Times New Roman\", \"Courier New\"}"),
            "#{font-family:\"Times New Roman\",\"Courier New\"}",
        );
        //str with single quotes
        assert_eq!(
            minify!("#{font-family:'Times New Roman', 'Courier New'}"),
            "#{font-family:'Times New Roman','Courier New'}",
        );
        //str with comment inside single quotes
        assert_eq!(
            minify!("#{font-family:'/*comment*/'}"),
            "#{font-family:'/*comment*/'}"
        );
    }

    #[test]
    fn unclosed_comments_quotes() {
        //should not panic
        assert_eq!(minify!("\""), "\"");
        assert_eq!(minify!("'"), "'");
        assert_eq!(minify!("/*"), "");
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
    fn hexcode_colors() {
        assert_eq!(minify!("#{color:#000}"), "#{color:#000}");
        assert_eq!(minify!("#{color:#abc}"), "#{color:#abc}");
        assert_eq!(minify!("#{color:#DEFG}"), "#{color:#DEFG}");
        assert_eq!(minify!("#{color:#aabbcc}"), "#{color:#abc}");
        assert_eq!(minify!("#{color:#aabbccdd}"), "#{color:#abcd}");
        assert_eq!(minify!("#{color:#aabbb}"), "#{color:#aabbb}");
        assert_eq!(minify!("#{color:#DDEEFFF}"), "#{color:#DDEEFFF}");
        assert_eq!(minify!("#{color:#aabbccddd}"), "#{color:#aabbccddd}");
    }

    #[test]
    fn rgbfunc_legacy_style() {
        assert_eq!(minify!("#{color:rgb(0, 0, 0)}"), "#{color:#000}");
        assert_eq!(minify!("#{color:rgb(255,255,254)}"), "#{color:#fffffe}");
        assert_eq!(minify!("#{color:rgb(255,255,255)}"), "#{color:#fff}");
        assert_eq!(minify!("#{color:rgb(0%, 0%, 0%)}"), "#{color:#000}");
        assert_eq!(minify!("#{color:rgb(1%, 2%, 3%)}"), "#{color:#030508}");
        assert_eq!(minify!("#{color:rgb(4%, 5%, 6%)}"), "#{color:#0a0d0f}");
        assert_eq!(minify!("#{color:rgb(7%, 8%, 9%)}"), "#{color:#121417}");
        assert_eq!(minify!("#{color:rgb(20%, 20%, 20%)}"), "#{color:#333}");
        assert_eq!(minify!("#{color:rgb(40%, 40%, 40%)}"), "#{color:#666}");
        assert_eq!(minify!("#{color:rgb(50%, 50%, 50%)}"), "#{color:#808080}");
        assert_eq!(minify!("#{color:rgb(60%, 60%, 60%)}"), "#{color:#999}");
        assert_eq!(minify!("#{color:rgb(80%, 80%, 80%)}"), "#{color:#ccc}");
        assert_eq!(minify!("#{color:rgb(100%, 100%, 100%)}"), "#{color:#fff}");
        assert_eq!(minify!("#{color:rgba(0%, 0%, 0%, 0)}"), "#{color:#0000}");
        assert_eq!(
            minify!("#{color:rgba(100%, 100%, 100%, 0.5)}"),
            "#{color:#ffffff80}"
        );
        assert_eq!(
            minify!("#{color:rgba(100%, 100%, 100%, 1)}"),
            "#{color:#fff}"
        );
        assert_eq!(
            minify!("#{color:rgba(80%, 80%, 80%, 0.8)}"),
            "#{color:#cccc}"
        );
    }

    #[test]
    fn rgbfunc_modern_style() {
        assert_eq!(minify!("#{color:rgb(0 0 0)}"), "#{color:#000}");
        assert_eq!(minify!("#{color:rgba(0 0 0)}"), "#{color:#000}");
        assert_eq!(minify!("#{color:rgb(0 100% 255)}"), "#{color:#0ff}");
        assert_eq!(minify!("#{color:rgb(0 0 0 / 0.5)}"), "#{color:#00000080}");
    }

    #[test]
    fn shakedown() {
        //include_str! inserts a newline at the end of the source file even though the file
        //doesn't contain it so we copy it here
        assert_eq!(
            // source.css from w3schools.com
            minify!("./tests/w3_source.css").to_string() + "\n",
            // expected.css produced by hand
            include_str!("./w3_expected.css").to_string()
        );
    }
}
