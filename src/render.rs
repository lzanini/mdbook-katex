//! Render KaTeX math block to HTML
use katex::{Error, Opts};

use super::*;

pub use {cfg::*, preprocess::*};

mod cfg;
mod preprocess;

/// Render a math block `item` into HTML following `opts`.
/// Wrap result in `<data>` tag if `extra_opts.include_src`.
#[instrument(skip(opts, extra_opts, display))]
pub fn render(item: &str, opts: Opts, extra_opts: ExtraOpts, display: bool) -> String {
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
        // if rendering fails, keep the unrendered equation
        Err(why) => {
            match why {
                Error::JsExecError(why) => {
                    warn!("Rendering failed, keeping the original content: {why}")
                }
                _ => error!(
                    ?why,
                    "Unexpected rendering failure, keeping the original content."
                ),
            }
            let delimiter = match display {
                true => &extra_opts.block_delimiter,
                false => &extra_opts.inline_delimiter,
            };
            rendered_content.push_str(&delimiter.left);
            rendered_content.push_str(item);
            rendered_content.push_str(&delimiter.right);
        }
    }

    rendered_content
}
