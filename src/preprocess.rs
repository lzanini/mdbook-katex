//! Preprocessing with KaTeX.
use std::borrow::Cow;

#[cfg(feature = "pre-render")]
use {crate::render::render, katex::Opts};

use mdbook::{
    book::Book,
    errors::Result,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    cfg::{get_config, KatexConfig},
    render::{escaping3, Render},
    scan::{Delimiter, Event, Scan},
};

/// Header that points to CDN for the KaTeX stylesheet.
pub const KATEX_HEADER: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css" integrity="sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X" crossorigin="anonymous">

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
            process_all_chapters_prerender(chapters, cfg, header, ctx)
        } else {
            process_all_chapters_escaping(chapters, cfg, header, ctx)
        };

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = contents.pop().expect("Chapter number mismatch.");
            }
        });
        Ok(book)
    }
}

/// Render Katex equations in a `Chapter` as HTML, and add the Katex CSS.
#[cfg(feature = "pre-render")]
pub fn process_chapter_prerender(
    raw_content: String,
    inline_opts: Opts,
    display_opts: Opts,
    stylesheet_header: String,
    extra_opts: ExtraOpts,
) -> String {
    get_render_tasks(&raw_content, &stylesheet_header, &extra_opts)
        .into_par_iter()
        .map(|rend| match rend {
            Render::Text(t) => t.into(),
            Render::InlineTask(item) => {
                render(item, inline_opts.clone(), extra_opts.clone()).into()
            }
            Render::DisplayTask(item) => {
                render(item, display_opts.clone(), extra_opts.clone()).into()
            }
        })
        .collect::<Vec<Cow<_>>>()
        .join("")
}

/// KaTeX reander
#[cfg(not(feature = "pre-render"))]
pub fn process_all_chapters_prerender(
    _: Vec<String>,
    _: KatexConfig,
    _: String,
    _: &PreprocessorContext,
) -> Vec<String> {
    panic!("Unable to pre-render. Please rebuild with the feature `pre-render`!")
}

/// KaTeX reander
#[cfg(feature = "pre-render")]
pub fn process_all_chapters_prerender(
    chapters: Vec<String>,
    cfg: KatexConfig,
    stylesheet_header: String,
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
                stylesheet_header.clone(),
                extra_opts.clone(),
            )
        })
        .collect();

    contents
}

/// KaTeX reander
pub fn process_all_chapters_escaping(
    chapters: Vec<String>,
    cfg: KatexConfig,
    stylesheet_header: String,
    _: &PreprocessorContext,
) -> Vec<String> {
    let extra_opts = cfg.build_extra_opts();

    let contents: Vec<_> = chapters
        .into_par_iter()
        .rev()
        .map(|raw_content| {
            process_chapter_escaping(raw_content, &extra_opts, stylesheet_header.clone())
        })
        .collect();

    contents
}

/// Escaping Katex equations.
pub fn process_chapter_escaping(
    raw_content: String,
    extra_opts: &ExtraOpts,
    stylesheet_header: String,
) -> String {
    get_render_tasks(&raw_content, &stylesheet_header, &extra_opts)
        .into_par_iter()
        .map(|rend| match rend {
            Render::Text(t) => t.into(),
            Render::InlineTask(item) => escaping3(item, &extra_opts.inline_delimiter).into(),
            Render::DisplayTask(item) => escaping3(item, &extra_opts.block_delimiter).into(),
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
