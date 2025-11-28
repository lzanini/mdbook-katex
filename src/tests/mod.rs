use super::*;

#[test]
fn test_name() {
    let pre = KatexProcessor;
    let preprocessor: &dyn Preprocessor = &pre;
    assert_eq!(preprocessor.name(), "katex")
}

#[test]
fn test_support_html() {
    let preprocessor = KatexProcessor;
    assert!(preprocessor.supports_renderer("html").unwrap());
    assert!(preprocessor.supports_renderer("other_renderer").unwrap())
}

mod escape;

#[cfg(feature = "pre-render")]
mod render;
