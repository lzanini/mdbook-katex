use mdbook::preprocess::Preprocessor;

use crate::{cfg::*, preprocess::*};

#[test]
fn test_name() {
    let pre = KatexProcessor;
    let preprocessor: &dyn Preprocessor = &pre;
    assert_eq!(preprocessor.name(), "katex")
}

#[test]
fn test_support_html() {
    let preprocessor = KatexProcessor;
    assert!(preprocessor.supports_renderer("html"));
    assert!(preprocessor.supports_renderer("other_renderer"))
}

fn test_render(raw_content: &str) -> (String, String) {
    let (stylesheet_header, mut rendered) =
        test_render_with_cfg(&[raw_content], KatexConfig::default());
    (stylesheet_header, rendered.pop().unwrap())
}

fn test_render_with_cfg(raw_contents: &[&str], cfg: KatexConfig) -> (String, Vec<String>) {
    let extra_opts = cfg.build_extra_opts();
    let stylesheet_header = KATEX_HEADER.to_owned();
    let rendered = raw_contents
        .iter()
        .map(|raw_content| process_chapter_escape(raw_content, &extra_opts, &stylesheet_header))
        .collect();
    (stylesheet_header, rendered)
}

#[test]
fn test_escape_without_math() {
    let raw_content = r"Some text, and more text.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_dollar_escape() {
    let raw_content = r"Some text, \$\$ and more text.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_escape_with_math() {
    let raw_content = r"A simple fomula, $\sum_{n=1}^\infty \frac{1}{n^2} = \frac{\pi^2}{6}$.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header
        + r"A simple fomula, $\\sum\_{n=1}^\\infty \\frac{1}{n^2} = \\frac{\\pi^2}{6}$.";
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_escape_underscore() {
    let raw_content = r"A simple `f_f_f`, f_f_f, f`f$f_$f_` fomula, $\sum_{n=1}^\infty\\$.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header
        + r"A simple `f_f_f`, f_f_f, f`f$f_$f_` fomula, $\\sum\_{n=1}^\\infty\\\\$.";
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_escape_vmatrix() {
    let raw_content = r"$$\begin{vmatrix}a&b\\c&d\end{vmatrix}$$";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + r"$$\\begin{vmatrix}a&b\\\\c&d\\end{vmatrix}$$";
    debug_assert_eq!(expected_output, rendered_content);
}
