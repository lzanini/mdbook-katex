//! Render KaTeX math block to HTML.
use katex::Opts;

use crate::preprocess::ExtraOpts;

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
pub fn render(item: &str, opts: Opts, extra_opts: ExtraOpts) -> String {
    let mut rendered_content = String::new();

    // try to render equation
    match katex::render_with_opts(item, opts) {
        Ok(rendered) => {
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
        }
        Err(err) => {
            eprintln!("mdbook-katex: Failed to render `{item}`: {err:?}.");
            // if rendering fails, keep the unrendered equation
            rendered_content.push_str(item)
        }
    }

    rendered_content
}
