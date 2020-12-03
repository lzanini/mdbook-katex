use super::*;

#[test]
fn test_name() {
    let preprocessor = KatexProcessor;
    assert_eq!(preprocessor.name(), "katex")
}

#[test]
fn test_support_html() {
    let preprocessor = KatexProcessor;
    assert!(preprocessor.supports_renderer("html"));
    assert!(!preprocessor.supports_renderer("other_renderer"))
}

fn mock_build_opts(macros: HashMap<String, String>) -> (katex::Opts, katex::Opts) {
    let inline_opts = katex::Opts::builder()
        .display_mode(false)
        .output_type(katex::OutputType::Html)
        .macros(macros.clone())
        .build()
        .unwrap();
    let display_opts = katex::Opts::builder()
        .display_mode(true)
        .output_type(katex::OutputType::Html)
        .macros(macros)
        .build()
        .unwrap();
    (inline_opts, display_opts)
}

#[test]
fn test_rendering_without_math() {
    let preprocessor = KatexProcessor;
    let macros = HashMap::new();
    let (inline_opts, display_opts) = mock_build_opts(macros);
    let raw_content = r"Some text, and more text.";
    let mut expected_output = katex_header();
    expected_output.push_str(raw_content);
    let rendered_content = preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_dollar_escaping() {
    let preprocessor = KatexProcessor;
    let macros = HashMap::new();
    let (inline_opts, display_opts) = mock_build_opts(macros);
    let raw_content = r"Some text, \$\$ and more text.";
    let mut expected_output = katex_header();
    expected_output.push_str(r"Some text, $$ and more text.");
    let rendered_content = preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_inline_rendering() {
    let preprocessor = KatexProcessor;
    let macros = HashMap::new();
    let (inline_opts, display_opts) = mock_build_opts(macros);
    let raw_content = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
    let mut expected_output = katex_header();
    expected_output.push_str("Some text, <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord\">∇</span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.68889em;vertical-align:0em;\"></span><span class=\"mord\"><span class=\"mord\"><span class=\"mord mathbb\">R</span></span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.664392em;\"><span style=\"top:-3.063em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">n</span></span></span></span></span></span></span></span></span></span></span>, and more text.");
    let rendered_content = preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_display_rendering() {
    let preprocessor = KatexProcessor;
    let macros = HashMap::new();
    let (inline_opts, display_opts) = mock_build_opts(macros);
    let raw_content = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
    let mut expected_output = katex_header();
    expected_output.push_str("Some text, <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord\">∇</span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.68889em;vertical-align:0em;\"></span><span class=\"mord\"><span class=\"mord\"><span class=\"mord mathbb\">R</span></span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.664392em;\"><span style=\"top:-3.063em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">n</span></span></span></span></span></span></span></span></span></span></span>, and more text.");
    let rendered_content = preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_macros_without_argument() {
    let preprocessor = KatexProcessor;
    let mut macros = HashMap::new();
    macros.insert(String::from(r"\grad"), String::from(r"\nabla"));
    let (inline_opts, display_opts) = mock_build_opts(macros);
    let raw_content_no_macro = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
    let raw_content_macro = r"Some text, $\grad f(x) \in \mathbb{R}^n$, and more text.";
    let rendered_content_macro =
        preprocessor.process_chapter(&raw_content_macro, &inline_opts, &display_opts);
    let rendered_content_no_macro =
        preprocessor.process_chapter(&raw_content_no_macro, &inline_opts, &display_opts);
    debug_assert_eq!(rendered_content_macro, rendered_content_no_macro);
}

#[test]
fn test_macros_with_argument() {
    let preprocessor = KatexProcessor;
    let mut macros = HashMap::new();
    macros.insert(String::from(r"\R"), String::from(r"\mathbb{R}^#1"));
    let (inline_opts, display_opts) = mock_build_opts(macros);
    let raw_content_no_macro = r"Some text, $\nabla f(x) \in \mathbb{R}^1$, and more text.";
    let raw_content_macro = r"Some text, $\nabla f(x) \in \R{1}$, and more text.";
    let rendered_content_macro =
        preprocessor.process_chapter(&raw_content_macro, &inline_opts, &display_opts);
    let rendered_content_no_macro =
        preprocessor.process_chapter(&raw_content_no_macro, &inline_opts, &display_opts);
    debug_assert_eq!(rendered_content_macro, rendered_content_no_macro);
}
