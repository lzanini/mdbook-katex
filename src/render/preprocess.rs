//! Preprocessing and pre-rendering with KaTeX.
use katex::Opts;

use super::*;

/// Render all Katex equations.
pub fn process_all_chapters_prerender(
    chapters: &Vec<String>,
    cfg: &KatexConfig,
    stylesheet_header: &str,
    ctx: &PreprocessorContext,
) -> Vec<String> {
    let extra_opts = cfg.build_extra_opts();
    let (inline_opts, display_opts) = cfg.build_opts(&ctx.root);

    let contents: Vec<_> = chapters
        .into_par_iter()
        .rev()
        .map(|raw_content| {
            process_chapter_prerender(
                raw_content,
                inline_opts.clone(),
                display_opts.clone(),
                stylesheet_header,
                &extra_opts,
            )
        })
        .collect();

    contents
}

/// Render Katex equations in a `Chapter` as HTML, and add the Katex CSS.
pub fn process_chapter_prerender(
    raw_content: &str,
    inline_opts: Opts,
    display_opts: Opts,
    stylesheet_header: &str,
    extra_opts: &ExtraOpts,
) -> String {
    get_render_tasks(raw_content, stylesheet_header, extra_opts)
        .into_par_iter()
        .map(|rend| match rend {
            Render::Text(t) => t.into(),
            Render::InlineTask(item) => {
                render(item, inline_opts.clone(), extra_opts.clone(), false).into()
            }
            Render::DisplayTask(item) => {
                render(item, display_opts.clone(), extra_opts.clone(), true).into()
            }
        })
        .collect::<Vec<Cow<_>>>()
        .join("")
}
