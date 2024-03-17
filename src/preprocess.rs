//! Preprocessing and escaping with KaTeX.
use std::borrow::Cow;

use mdbook::{
    book::Book,
    errors::Result,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    cfg::{get_config, KatexConfig},
    escape::{escape_math_with_delimiter, Render},
    scan::{Delimiter, Event, Scan},
};

/// When `pre-render` is called but not enabled.
#[cfg(not(feature = "pre-render"))]
pub fn process_all_chapters_prerender(
    _: &Vec<String>,
    _: &KatexConfig,
    _: &str,
    _: &PreprocessorContext,
) -> Vec<String> {
    panic!("Pre-render is unavailable because this `mdbook-katex` program does not have the `pre-render` feature enabled, only escaping mode is available, and you can set `pre-render = false` to enable it. If you do need `pre-render` mode, you need to add the `pre-render` feature and recompile. See the README at <https://github.com/lzanini/mdbook-katex/blob/master/README.md>.")
}

#[cfg(feature = "pre-render")]
use crate::preprocess_render::process_all_chapters_prerender;

/// Header that points to CDN for the KaTeX stylesheet.
pub const KATEX_HEADER: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.css">

"#;

/// Extra options for the KaTeX preprocessor.
#[derive(Clone, Debug)]
pub struct ExtraOpts {
    /// Path to macro file.
    pub include_src: bool,
    /// Delimiter for math display block.
    pub block_delimiter: Delimiter,
    /// Delimiter for math inline block.
    pub inline_delimiter: Delimiter,
}

/// KaTeX `mdbook::preprocess::Proprecessor` for mdBook.
pub struct KatexProcessor;

// preprocessor to inject rendered katex blocks and stylesheet
impl Preprocessor for KatexProcessor {
    fn name(&self) -> &str {
        "katex"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // parse TOML config
        let cfg = get_config(&ctx.config)?;
        let header = if cfg.no_css { "" } else { KATEX_HEADER }.to_owned();
        let mut chapters = Vec::with_capacity(book.sections.len());
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapters.push(chapter.content.clone());
            }
        });

        let mut contents = if cfg.pre_render {
            process_all_chapters_prerender(&chapters, &cfg, &header, ctx)
        } else {
            process_all_chapters_escape(&chapters, &cfg, &header, ctx)
        };

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = contents.pop().expect("Chapter number mismatch.");
            }
        });
        Ok(book)
    }
}

/// Escape all Katex equations.
pub fn process_all_chapters_escape(
    chapters: &Vec<String>,
    cfg: &KatexConfig,
    stylesheet_header: &str,
    _: &PreprocessorContext,
) -> Vec<String> {
    let extra_opts = cfg.build_extra_opts();

    let contents: Vec<_> = chapters
        .into_par_iter()
        .rev()
        .map(|raw_content| process_chapter_escape(raw_content, &extra_opts, stylesheet_header))
        .collect();

    contents
}

/// Escape Katex equations.
pub fn process_chapter_escape(
    raw_content: &str,
    extra_opts: &ExtraOpts,
    stylesheet_header: &str,
) -> String {
    get_render_tasks(raw_content, stylesheet_header, extra_opts)
        .into_par_iter()
        .map(|rend| match rend {
            Render::Text(t) => t.into(),
            Render::InlineTask(item) => {
                escape_math_with_delimiter(item, &extra_opts.inline_delimiter).into()
            }
            Render::DisplayTask(item) => {
                escape_math_with_delimiter(item, &extra_opts.block_delimiter).into()
            }
        })
        .collect::<Vec<Cow<_>>>()
        .join("")
}

/// Find all the `Render` tasks in `raw_content`.
pub fn get_render_tasks<'a>(
    raw_content: &'a str,
    stylesheet_header: &'a str,
    extra_opts: &ExtraOpts,
) -> Vec<Render<'a>> {
    let scan = Scan::new(
        raw_content,
        &extra_opts.block_delimiter,
        &extra_opts.inline_delimiter,
    );

    let mut rendering = Vec::new();
    rendering.push(Render::Text(stylesheet_header));

    let mut checkpoint = 0;
    for event in scan {
        match event {
            Event::Begin(begin) => checkpoint = begin,
            Event::TextEnd(end) => rendering.push(Render::Text(&raw_content[checkpoint..end])),
            Event::InlineEnd(end) => {
                rendering.push(Render::InlineTask(&raw_content[checkpoint..end]));
                checkpoint = end;
            }
            Event::BlockEnd(end) => {
                rendering.push(Render::DisplayTask(&raw_content[checkpoint..end]));
                checkpoint = end;
            }
        }
    }

    if raw_content.len() > checkpoint {
        rendering.push(Render::Text(&raw_content[checkpoint..raw_content.len()]));
    }
    rendering
}
