//! Preprocessing with KaTeX.
use std::borrow::Cow;

use katex::Opts;
use mdbook::{
    book::Book,
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    cfg::get_config,
    render::{render, Render},
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

    #[tokio::main]
    async fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // parse TOML config
        let cfg = get_config(&ctx.config)?;
        let (inline_opts, display_opts, extra_opts) = cfg.build_opts(&ctx.root);
        let header = if cfg.no_css { "" } else { KATEX_HEADER }.to_owned();
        let mut chapters = Vec::with_capacity(book.sections.len());
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapters.push(chapter.content.clone());
            }
        });
        let mut contents: Vec<_> = chapters
            .into_par_iter()
            .rev()
            .map(|raw_content| {
                process_chapter(
                    raw_content,
                    inline_opts.clone(),
                    display_opts.clone(),
                    header.clone(),
                    extra_opts.clone(),
                )
            })
            .collect();
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if chapter.path.is_some() {
                    chapter.content = contents.pop().expect("Chapter number mismatch.");
                }
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "markdown"
    }
}

/// Render Katex equations in a `Chapter` as HTML, and add the Katex CSS.
pub fn process_chapter(
    raw_content: String,
    inline_opts: Opts,
    display_opts: Opts,
    stylesheet_header: String,
    extra_opts: ExtraOpts,
) -> String {
    let mut scan = Scan::new(
        &raw_content,
        &extra_opts.block_delimiter,
        &extra_opts.inline_delimiter,
    );
    scan.run();

    let mut rendering = Vec::with_capacity(scan.events.len() / 2 + 5);
    rendering.push(Render::Text(&stylesheet_header));

    let mut checkpoint = 0;
    let events = scan.events.iter();
    for event in events {
        match *event {
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

    if raw_content.len() - 1 > checkpoint {
        rendering.push(Render::Text(&raw_content[checkpoint..raw_content.len()]));
    }

    let rendered: Vec<Cow<str>> = rendering
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
        .collect();
    rendered.join("")
}
