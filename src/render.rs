//! Render KaTeX math block to HTML.
use katex::Opts;
use tokio::task::JoinHandle;

use crate::preprocess::ExtraOpts;

/// A render job for `process_chapter`.
pub enum Render {
    /// No need to render.
    Text(String),
    /// A running render job for a math block.
    Task(JoinHandle<String>),
}

/// Render a math block `item` into HTML following `opts`.
/// Wrap result in `<data>` tag if `extra_opts.include_src`.
pub async fn render(item: String, opts: Opts, extra_opts: ExtraOpts) -> String {
    let mut rendered_content = String::new();

    // try to render equation
    if let Ok(rendered) = katex::render_with_opts(&item, opts) {
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
        rendered_content.push_str(&item)
    }

    rendered_content
}
