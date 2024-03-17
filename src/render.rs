//! Render KaTeX math block to HTML

use {crate::preprocess::ExtraOpts, katex::Opts};

/// Render a math block `item` into HTML following `opts`.
/// Wrap result in `<data>` tag if `extra_opts.include_src`.
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
