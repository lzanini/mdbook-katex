//! Render KaTeX math block to HTML

use crate::scan::Delimiter;
#[cfg(feature = "pre-render")]
use {crate::preprocess::ExtraOpts, katex::Opts};

/// A render job for `process_chapter`.
pub enum Render<'a> {
    /// No need to render.
    Text(&'a str),
    /// A render task for a math inline block.
    InlineTask(&'a str),
    /// A render task for a math display block.
    DisplayTask(&'a str),
}

/// Render a math block `item` into HTML following `opts`.
/// Wrap result in `<data>` tag if `extra_opts.include_src`.
#[cfg(feature = "pre-render")]
pub fn render(item: &str, opts: Opts, extra_opts: ExtraOpts) -> String {
    let mut rendered_content = String::new();

    // try to render equation
    if let Ok(rendered) = katex::render_with_opts(item, opts) {
        let rendered = rendered.replace('\n', " ");
        if extra_opts.include_src {
            // Wrap around with `data.katex-src` tag.
            rendered_content.push_str(r#"<data class="katex-src" value=""#);
            rendered_content.push_str(&item.replace('"', r#"\""#).replace('\n', r"&#10;"));
            rendered_content.push_str(r#"">"#);
            rendered_content.push_str(&rendered);
            rendered_content.push_str(r"</data>");
        } else {
            rendered_content.push_str(&rendered);
        }
    // if rendering fails, keep the unrendered equation
    } else {
        rendered_content.push_str(item)
    }

    rendered_content
}

/// Render a math block `item` into HTML following `opts`.
pub fn escaping3(code: &str, delimiter: &Delimiter) -> String {
    let mut result = String::new();
    escaping(&delimiter.left, &mut result);
    escaping(code, &mut result);
    escaping(&delimiter.right, &mut result);
    result
}

/// Render a math block `item` into HTML following `opts`.
pub fn escaping(code: &str, result: &mut String) {
    for c in code.chars() {
        match c {
            '_' => {
                result.push_str("\\_");
            }
            '*' => {
                result.push_str("\\*");
            }
            '\\' => {
                result.push_str("\\\\");
            }
            _ => {
                result.push(c);
            }
        }
    }
}
