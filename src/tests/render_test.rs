use crate::{cfg::*, preprocess::*, scan::*};

use mdbook::preprocess::Preprocessor;
use std::{collections::HashMap, path::PathBuf, str::FromStr};

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
    let (stylesheet_header, mut rendered) = test_render_with_macro(&[raw_content], HashMap::new());
    (stylesheet_header, rendered.pop().unwrap())
}

fn test_render_with_macro(
    raw_contents: &[&str],
    macros: HashMap<String, String>,
) -> (String, Vec<String>) {
    test_render_with_cfg(raw_contents, macros, KatexConfig::default())
}

fn test_render_with_cfg(
    raw_contents: &[&str],
    macros: HashMap<String, String>,
    cfg: KatexConfig,
) -> (String, Vec<String>) {
    let (inline_opts, display_opts) = cfg.build_opts_from_macros(macros);
    let extra_opts = cfg.build_extra_opts();
    let stylesheet_header = KATEX_HEADER.to_owned();
    let rendered = raw_contents
        .iter()
        .map(|raw_content| {
            process_chapter_prerender(
                raw_content,
                inline_opts.clone(),
                display_opts.clone(),
                &stylesheet_header,
                &extra_opts,
            )
        })
        .collect();
    (stylesheet_header, rendered)
}

#[test]
fn test_rendering_without_math() {
    let raw_content = r"Some text, and more text.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_dollar_escaping() {
    let raw_content = r"Some text, \$\$ and more text.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_dollar_escaping_inside_expr() {
    let raw_content = r"We randomly assign: $r \xleftarrow{\$} G $.";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header +
    "We randomly assign: <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1.158em;vertical-align:-0.011em;\"></span><span class=\"mord mathnormal\" style=\"margin-right:0.02778em;\">r</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span><span class=\"mrel x-arrow\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.147em;\"><span style=\"top:-3.322em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight x-arrow-pad\"><span class=\"mord mtight\"><span class=\"mord mtight\">$</span></span></span></span><span class=\"svg-align\" style=\"top:-2.689em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"hide-tail\" style=\"height:0.522em;min-width:1.469em;\"><svg xmlns=\"http://www.w3.org/2000/svg\" width='400em' height='0.522em' viewBox='0 0 400000 522' preserveAspectRatio='xMinYMin slice'><path d='M400000 241H110l3-3c68.7-52.7 113.7-120  135-202 4-14.7 6-23 6-25 0-7.3-7-11-21-11-8 0-13.2.8-15.5 2.5-2.3 1.7-4.2 5.8 -5.5 12.5-1.3 4.7-2.7 10.3-4 17-12 48.7-34.8 92-68.5 130S65.3 228.3 18 247 c-10 4-16 7.7-18 11 0 8.7 6 14.3 18 17 47.3 18.7 87.8 47 121.5 85S196 441.3 208  490c.7 2 1.3 5 2 9s1.2 6.7 1.5 8c.3 1.3 1 3.3 2 6s2.2 4.5 3.5 5.5c1.3 1 3.3  1.8 6 2.5s6 1 10 1c14 0 21-3.7 21-11 0-2-2-10.3-6-25-20-79.3-65-146.7-135-202  l-3-3h399890zM100 241v40h399900v-40z'/></svg></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.011em;\"><span></span></span></span></span></span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.6833em;\"></span><span class=\"mord mathnormal\">G</span></span></span></span>.";
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_inline_rendering() {
    let (stylesheet_header, rendered_content) =
        test_render(r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.");
    let expected_output=stylesheet_header+"Some text, <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord\">∇</span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.6889em;\"></span><span class=\"mord\"><span class=\"mord mathbb\">R</span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.6644em;\"><span style=\"top:-3.063em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">n</span></span></span></span></span></span></span></span></span></span></span>, and more text.";
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_display_rendering() {
    let (stylesheet_header, rendered_content) =
        test_render(r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.");
    let expected_output=stylesheet_header+"Some text, <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord\">∇</span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.6889em;\"></span><span class=\"mord\"><span class=\"mord mathbb\">R</span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.6644em;\"><span style=\"top:-3.063em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">n</span></span></span></span></span></span></span></span></span></span></span>, and more text.";
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_macros_without_argument() {
    let mut macros = HashMap::new();
    macros.insert(String::from(r"\grad"), String::from(r"\nabla"));
    let raw_content_no_macro = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
    let raw_content_macro = r"Some text, $\grad f(x) \in \mathbb{R}^n$, and more text.";
    let (_, rendered) = test_render_with_macro(&[raw_content_macro, raw_content_no_macro], macros);
    debug_assert_eq!(rendered[0], rendered[1]);
}

#[test]
fn test_macros_with_argument() {
    let mut macros = HashMap::new();
    macros.insert(String::from(r"\R"), String::from(r"\mathbb{R}^#1"));
    let raw_content_no_macro = r"Some text, $\nabla f(x) \in \mathbb{R}^1$, and more text.";
    let raw_content_macro = r"Some text, $\nabla f(x) \in \R{1}$, and more text.";
    let (_, rendered) = test_render_with_macro(&[raw_content_macro, raw_content_no_macro], macros);
    debug_assert_eq!(rendered[0], rendered[1]);
}

#[test]
fn test_macro_file_loading() {
    let cfg_str = r#"
    [book]
    src = "src"

    [preprocessor.katex]
    macros = "macros.txt"
    "#;

    let book_cfg = mdbook::config::Config::from_str(cfg_str).unwrap();
    let cfg = get_config(&book_cfg).unwrap();

    debug_assert_eq!(
        get_macro_path(PathBuf::from("book"), &cfg.macros),
        Some(PathBuf::from("book/macros.txt")) // We supply a root, just like the preproccessor context does
    );
}

#[test]
fn test_rendering_table_with_math() {
    let raw_content = r"| Syntax | Description |
| --- | ----------- |
| $\vec{a}$ | Title |
| Paragraph | Text |";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(
        expected_output.lines().count(),
        rendered_content.lines().count()
    );
}

#[test]
fn test_rendering_delimiter_in_code_block() {
    let raw_content = r"``` $\omega$ ```";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_rendering_delimiter_in_inline_code() {
    let raw_content = r"`$\omega$`";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + raw_content;
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_escaping_backtick() {
    let raw_content = r"\`$\omega$\`";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    let expected_output = stylesheet_header + "\\`<span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:0.4306em;\"></span><span class=\"mord mathnormal\" style=\"margin-right:0.03588em;\">ω</span></span></span></span>\\`";
    debug_assert_eq!(expected_output, rendered_content);
}

#[test]
fn test_include_src() {
    let raw_content = r"Define $f(x)$:

$$
f(x)=x^2\\

x\in\R
$$";
    let (stylesheet_header, rendered_content) = test_render_with_cfg(
        &[raw_content],
        HashMap::new(),
        KatexConfig {
            include_src: true,
            ..KatexConfig::default()
        },
    );
    debug_assert_eq!(stylesheet_header + "Define <data class=\"katex-src\" value=\"f(x)\"><span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span></span></span></span></data>:\n\n<data class=\"katex-src\" value=\"&#10;f(x)=x^2\\\\&#10;&#10;x\\in\\R&#10;\"><span class=\"katex-display\"><span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span><span class=\"mrel\">=</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.8641em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">x</span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.8641em;\"><span style=\"top:-3.113em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mtight\">2</span></span></span></span></span></span></span></span></span><span class=\"mspace newline\"></span><span class=\"base\"><span class=\"strut\" style=\"height:0.5782em;vertical-align:-0.0391em;\"></span><span class=\"mord mathnormal\">x</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.6889em;\"></span><span class=\"mord mathbb\">R</span></span></span></span></span></data>", rendered_content[0]);
}

#[test]
fn test_fenced_code() {
    let raw_content = r"`\` and `` ` `` $\Leftarrow$
```
`\` and `` ` ``
```
while ` ``` ` and ````` ```` ````` $\Leftarrow$
``````
` ``` ` and ````` ```` `````
``````
$$
\Uparrow
$$";
    let (stylesheet_header, rendered_content) =
        test_render_with_cfg(&[raw_content], HashMap::new(), KatexConfig::default());
    debug_assert_eq!(
        stylesheet_header +
        "`\\` and `` ` `` <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:0.3669em;\"></span><span class=\"mrel\">⇐</span></span></span></span>\n```\n`\\` and `` ` ``\n```\nwhile ` ``` ` and ````` ```` ````` <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:0.3669em;\"></span><span class=\"mrel\">⇐</span></span></span></span>\n``````\n` ``` ` and ````` ```` `````\n``````\n<span class=\"katex-display\"><span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:0.8889em;vertical-align:-0.1944em;\"></span><span class=\"mrel\">⇑</span></span></span></span></span>",
        rendered_content[0]
    );
}

#[test]
fn test_inline_rendering_w_custom_delimiter() {
    let raw_content = r"These $\(a\times b\) are from
\[
\int_0^abdx
\]";
    let (stylesheet_header, rendered_content) = test_render_with_cfg(
        &[raw_content],
        HashMap::new(),
        KatexConfig {
            inline_delimiter: Delimiter {
                left: r"\(".into(),
                right: r"\)".into(),
            },
            block_delimiter: Delimiter {
                left: r"\[".into(),
                right: r"\]".into(),
            },
            ..KatexConfig::default()
        },
    );
    let expected_output = stylesheet_header + "These $<span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:0.6667em;vertical-align:-0.0833em;\"></span><span class=\"mord mathnormal\">a</span><span class=\"mspace\" style=\"margin-right:0.2222em;\"></span><span class=\"mbin\">×</span><span class=\"mspace\" style=\"margin-right:0.2222em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.6944em;\"></span><span class=\"mord mathnormal\">b</span></span></span></span> are from\n<span class=\"katex-display\"><span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:2.3262em;vertical-align:-0.9119em;\"></span><span class=\"mop\"><span class=\"mop op-symbol large-op\" style=\"margin-right:0.44445em;position:relative;top:-0.0011em;\">∫</span><span class=\"msupsub\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.4143em;\"><span style=\"top:-1.7881em;margin-left:-0.4445em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mtight\">0</span></span></span><span style=\"top:-3.8129em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">a</span></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.9119em;\"><span></span></span></span></span></span></span><span class=\"mspace\" style=\"margin-right:0.1667em;\"></span><span class=\"mord mathnormal\">b</span><span class=\"mord mathnormal\">d</span><span class=\"mord mathnormal\">x</span></span></span></span></span>";
    debug_assert_eq!(expected_output, rendered_content[0]);
}

#[test]
#[cfg(any(
    target_os = "macos",
    all(unix, target_arch = "x86_64"),
    all(windows, target_env = "gnu")
))]
fn test_katex_rendering_vmatrix() {
    use crate::cfg::KatexConfig;

    let math_expr = r"\begin{vmatrix}a&b\\c&d\end{vmatrix}";
    let cfg = KatexConfig::default();
    let (_, display_opts) = cfg.build_opts_from_macros(HashMap::new());
    let _ = katex::render_with_opts(math_expr, display_opts).unwrap();
}

#[test]
#[cfg(any(
    target_os = "macos",
    all(unix, target_arch = "x86_64"),
    all(windows, target_env = "gnu")
))]
fn test_rendering_vmatrix() {
    let raw_content = r"$$\begin{vmatrix}a&b\\c&d\end{vmatrix}$$";
    let (stylesheet_header, rendered_content) = test_render(raw_content);
    debug_assert_eq!(
        stylesheet_header+
        "<span class=\"katex-display\"><span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:2.4em;vertical-align:-0.95em;\"></span><span class=\"minner\"><span class=\"mopen\"><span class=\"delimsizing mult\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.45em;\"><span class=\"pstrut\" style=\"height:4.4em;\"></span><span style=\"width:0.333em;height:2.400em;\"><svg xmlns=\"http://www.w3.org/2000/svg\" width='0.333em' height='2.400em' viewBox='0 0 333 2400'><path d='M145 15 v585 v1200 v585 c2.667,10,9.667,15,21,15 c10,0,16.667,-5,20,-15 v-585 v-1200 v-585 c-2.667,-10,-9.667,-15,-21,-15 c-10,0,-16.667,5,-20,15z M188 15 H145 v585 v1200 v585 h43z'/></svg></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span></span><span class=\"mord\"><span class=\"mtable\"><span class=\"col-align-c\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.61em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">a</span></span></span><span style=\"top:-2.41em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">c</span></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span><span class=\"arraycolsep\" style=\"width:0.5em;\"></span><span class=\"arraycolsep\" style=\"width:0.5em;\"></span><span class=\"col-align-c\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.61em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">b</span></span></span><span style=\"top:-2.41em;\"><span class=\"pstrut\" style=\"height:3em;\"></span><span class=\"mord\"><span class=\"mord mathnormal\">d</span></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span></span></span><span class=\"mclose\"><span class=\"delimsizing mult\"><span class=\"vlist-t vlist-t2\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:1.45em;\"><span style=\"top:-3.45em;\"><span class=\"pstrut\" style=\"height:4.4em;\"></span><span style=\"width:0.333em;height:2.400em;\"><svg xmlns=\"http://www.w3.org/2000/svg\" width='0.333em' height='2.400em' viewBox='0 0 333 2400'><path d='M145 15 v585 v1200 v585 c2.667,10,9.667,15,21,15 c10,0,16.667,-5,20,-15 v-585 v-1200 v-585 c-2.667,-10,-9.667,-15,-21,-15 c-10,0,-16.667,5,-20,15z M188 15 H145 v585 v1200 v585 h43z'/></svg></span></span></span><span class=\"vlist-s\">\u{200b}</span></span><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.95em;\"><span></span></span></span></span></span></span></span></span></span></span></span>",
        rendered_content
    );
}
