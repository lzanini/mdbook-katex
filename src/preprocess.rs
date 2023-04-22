//! Preprocessing with KaTeX.
use std::collections::VecDeque;

use katex::Opts;
use mdbook::{
    book::Book,
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use tokio::spawn;

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
        let mut tasks = Vec::with_capacity(book.sections.len());
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                tasks.push(spawn(process_chapter(
                    chapter.content.clone(),
                    inline_opts.clone(),
                    display_opts.clone(),
                    header.clone(),
                    extra_opts.clone(),
                )));
            }
        });
        let mut contents = VecDeque::with_capacity(tasks.len());
        for task in tasks {
            contents.push_back(task.await.expect("A tokio task panicked."));
        }
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if chapter.path.is_some() {
                    chapter.content = contents.pop_front().expect("Chapter number mismatch.");
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
pub async fn process_chapter(
    raw_content: String,
    inline_opts: Opts,
    display_opts: Opts,
    stylesheet_header: String,
    extra_opts: ExtraOpts,
) -> String {
    let mut rendering = Vec::new();
    rendering.push(Render::Text(stylesheet_header.to_owned()));

    let mut scan = Scan::new(
        &raw_content,
        &extra_opts.block_delimiter,
        &extra_opts.inline_delimiter,
    );
    scan.run();

    let mut checkpoint = 0;
    let events = scan.events.iter();
    for event in events {
        match *event {
            Event::Begin(begin) => checkpoint = begin,
            Event::TextEnd(end) => {
                rendering.push(Render::Text((&raw_content[checkpoint..end]).into()))
            }
            Event::InlineEnd(end) => {
                let inline_feed = (&raw_content[checkpoint..end]).into();
                let inline_block =
                    spawn(render(inline_feed, inline_opts.clone(), extra_opts.clone()));
                rendering.push(Render::Task(inline_block));
                checkpoint = end;
            }
            Event::BlockEnd(end) => {
                let block_feed = (&raw_content[checkpoint..end]).into();
                let block = spawn(render(block_feed, display_opts.clone(), extra_opts.clone()));
                rendering.push(Render::Task(block));
                checkpoint = end;
            }
        }
    }

    if raw_content.len() - 1 > checkpoint {
        rendering.push(Render::Text(
            (&raw_content[checkpoint..raw_content.len()]).into(),
        ));
    }
    let mut rendered = Vec::with_capacity(rendering.len());
    for r in rendering {
        rendered.push(match r {
            Render::Text(t) => t,
            Render::Task(t) => t.await.expect("A tokio task panicked."),
        });
    }
    rendered.join("")
}
