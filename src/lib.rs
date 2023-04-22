#![deny(missing_docs)]
//! Preprocess math blocks using KaTeX for mdBook.
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::vec::Vec;

use katex::Opts;

use serde_derive::{Deserialize, Serialize};

use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

use tokio::spawn;
use tokio::task::JoinHandle;

/// Header that points to CDN for the KaTeX stylesheet.
pub const KATEX_HEADER: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css" integrity="sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X" crossorigin="anonymous">

"#;

/// A pair of strings are delimiters.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Delimiter {
    /// Left delimiter.
    pub left: String,
    /// Right delimiter.
    pub right: String,
}

impl Delimiter {
    /// Same left and right `delimiter`.
    pub fn same(delimiter: String) -> Self {
        Self {
            left: delimiter.clone(),
            right: delimiter,
        }
    }

    /// The first byte of the left delimiter.
    pub fn first(&self) -> u8 {
        self.left.as_bytes()[0]
    }

    /// Whether `to_match` matches the left delimiter.
    pub fn match_left(&self, to_match: &[u8]) -> bool {
        if self.left.len() > to_match.len() {
            return false;
        }
        for (we, they) in self.left.as_bytes().iter().zip(to_match) {
            if we != they {
                return false;
            }
        }
        true
    }
}

/// Configuration for KaTeX preprocessor,
/// including options for `katex-rs` and feature options.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct KatexConfig {
    // options for the katex-rust crate
    /// KaTeX output type.
    pub output: String,
    /// Whether to have `\tags` rendered on the left instead of the right.
    pub leqno: bool,
    /// Whether to make display math flush left.
    pub fleqn: bool,
    /// Whether to let KaTeX throw a ParseError for invalid LaTeX.
    pub throw_on_error: bool,
    /// Color used for invalid LaTeX.
    pub error_color: String,
    /// Specifies a minimum thickness, in ems.
    pub min_rule_thickness: f64,
    /// Max size for user-specified sizes.
    pub max_size: f64,
    /// Limit the number of macro expansions to the specified number.
    pub max_expand: i32,
    /// Whether to trust users' input.
    pub trust: bool,
    // other options
    /// Do not inject KaTeX CSS headers.
    pub no_css: bool,
    /// Include math source in rendered HTML.
    pub include_src: bool,
    /// Path to macro file.
    pub macros: Option<String>,
    /// Delimiter for math display block.
    pub block_delimiter: Delimiter,
    /// Delimiter for math inline block.
    pub inline_delimiter: Delimiter,
}

impl Default for KatexConfig {
    fn default() -> KatexConfig {
        KatexConfig {
            // default options for the katex-rust crate
            // uses defaults specified in: https://katex.org/docs/options.html
            output: "html".into(),
            leqno: false,
            fleqn: false,
            throw_on_error: true,
            error_color: String::from("#cc0000"),
            min_rule_thickness: -1.0,
            max_size: f64::INFINITY,
            max_expand: 1000,
            trust: false,
            // other options
            no_css: false,
            include_src: false,
            macros: None,
            block_delimiter: Delimiter::same("$$".into()),
            inline_delimiter: Delimiter::same("$".into()),
        }
    }
}

impl KatexConfig {
    /// Configured output type.
    /// Defaults to `Html`, can also be `Mathml` or `HtmlAndMathml`.
    pub fn output_type(&self) -> katex::OutputType {
        match self.output.as_str() {
            "html" => katex::OutputType::Html,
            "mathml" => katex::OutputType::Mathml,
            "htmlAndMathml" => katex::OutputType::HtmlAndMathml,
            other => {
                eprintln!(
"[preprocessor.katex]: `{other}` is not a valid choice for `output`! Please check your `book.toml`.
Defaulting to `html`. Other valid choices for output are `mathml` and `htmlAndMathml`."
                );
                katex::OutputType::Html
            }
        }
    }

    /// From `root`, load macros and generate configuration options
    /// `(inline_opts, display_opts, extra_opts)`.
    pub fn build_opts<P>(&self, root: P) -> (katex::Opts, katex::Opts, ExtraOpts)
    where
        P: AsRef<Path>,
    {
        // load macros as a HashMap
        let macros = load_macros(root, &self.macros);

        self.build_opts_from_macros(macros)
    }

    /// Given `macros`, generate `(inline_opts, display_opts, extra_opts)`.
    pub fn build_opts_from_macros(
        &self,
        macros: HashMap<String, String>,
    ) -> (katex::Opts, katex::Opts, ExtraOpts) {
        let mut configure_katex_opts = katex::Opts::builder();
        configure_katex_opts
            .output_type(self.output_type())
            .leqno(self.leqno)
            .fleqn(self.fleqn)
            .throw_on_error(self.throw_on_error)
            .error_color(self.error_color.clone())
            .macros(macros)
            .min_rule_thickness(self.min_rule_thickness)
            .max_size(self.max_size)
            .max_expand(self.max_expand)
            .trust(self.trust);
        // inline rendering options
        let inline_opts = configure_katex_opts
            .clone()
            .display_mode(false)
            .build()
            .unwrap();
        // display rendering options
        let display_opts = configure_katex_opts.display_mode(true).build().unwrap();
        let extra_opts = ExtraOpts {
            include_src: self.include_src,
            block_delimiter: self.block_delimiter.clone(),
            inline_delimiter: self.inline_delimiter.clone(),
        };
        (inline_opts, display_opts, extra_opts)
    }
}

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
        let mut raw_contents = Vec::new();
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if chapter.path.is_some() {
                    raw_contents.push(chapter.content.clone());
                }
            }
        });
        let mut tasks = Vec::with_capacity(raw_contents.len());
        for content in raw_contents {
            let header = if cfg.no_css { "" } else { KATEX_HEADER }.into();
            tasks.push(spawn(process_chapter(
                content,
                inline_opts.clone(),
                display_opts.clone(),
                header,
                extra_opts.clone(),
            )));
        }
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

/// Load macros from `root`/`macros_path` into a `HashMap`.
fn load_macros<P>(root: P, macros_path: &Option<String>) -> HashMap<String, String>
where
    P: AsRef<Path>,
{
    // load macros as a HashMap
    let mut map = HashMap::new();
    if let Some(path) = get_macro_path(root, macros_path) {
        let macro_str = load_as_string(&path);
        for couple in macro_str.split('\n') {
            // only consider lines starting with a backslash
            if let Some('\\') = couple.chars().next() {
                let couple: Vec<&str> = couple.splitn(2, ':').collect();
                map.insert(String::from(couple[0]), String::from(couple[1]));
            }
        }
    }
    map
}

/// A render job for `process_chapter`.
pub enum Render {
    /// No need to render.
    Text(String),
    /// A running render job for a math block.
    Task(JoinHandle<String>),
}

/// Render Katex equations in a `Chapter` as HTML, and add the Katex CSS.
async fn process_chapter(
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

/// An event for parsing in a Markdown file.
#[derive(Debug)]
pub enum Event {
    /// A beginning of text or math block.
    Begin(usize),
    /// An end of a text block.
    TextEnd(usize),
    /// An end of an inline math block.
    InlineEnd(usize),
    /// An end of a display math block.
    BlockEnd(usize),
}

/// Scanner for text to identify block and inline math `Event`s.
#[derive(Debug)]
pub struct Scan<'a> {
    string: &'a str,
    bytes: &'a [u8],
    index: usize,
    /// Block and inline math `Event`s.
    pub events: Vec<Event>,
    block_delimiter: &'a Delimiter,
    inline_delimiter: &'a Delimiter,
}

impl<'a> Scan<'a> {
    /// Set up a `Scan` for `string` with given delimiters.
    pub fn new(
        string: &'a str,
        block_delimiter: &'a Delimiter,
        inline_delimiter: &'a Delimiter,
    ) -> Self {
        Self {
            string,
            bytes: string.as_bytes(),
            index: 0,
            events: Vec::new(),
            block_delimiter,
            inline_delimiter,
        }
    }

    /// Scan, identify and store all `Event`s in `self.events`.
    pub fn run(&mut self) {
        while let Ok(()) = self.process_byte() {}
    }

    /// Get byte currently pointed to. Returns `Err(())` if out of bound.
    fn get_byte(&self) -> Result<u8, ()> {
        self.bytes.get(self.index).map(|b| b.to_owned()).ok_or(())
    }

    /// Increment index.
    fn inc(&mut self) {
        self.index += 1;
    }

    /// Scan one byte, proceed process based on the byte.
    /// - Start of delimiter => call `process_delimit`.
    /// - `\` => skip one byte.
    /// - `` ` `` => call `process_backtick`.
    /// Return `Err(())` if no more bytes to process.
    fn process_byte(&mut self) -> Result<(), ()> {
        let byte = self.get_byte()?;
        self.inc();
        match byte {
            b if b == self.block_delimiter.first() || b == self.inline_delimiter.first() => {
                self.index -= 1;
                if self.block_delimiter.match_left(&self.bytes[self.index..]) {
                    self.process_delimit(false)?;
                } else if self.inline_delimiter.match_left(&self.bytes[self.index..]) {
                    self.process_delimit(true)?;
                } else {
                    self.inc();
                }
            }
            b'\\' => {
                self.inc();
            }
            b'`' => self.process_backtick()?,
            _ => (),
        }
        Ok(())
    }

    /// Fully skip a backtick-delimited code block.
    /// Guaranteed to match the number of backticks in delimiters.
    /// Return `Err(())` if no more bytes to process.
    fn process_backtick(&mut self) -> Result<(), ()> {
        let mut n_back_ticks = 1;
        loop {
            let byte = self.get_byte()?;
            if byte == b'`' {
                self.inc();
                n_back_ticks += 1;
            } else {
                break;
            }
        }
        loop {
            self.index += self.string[self.index..]
                .find(&"`".repeat(n_back_ticks))
                .ok_or(())?
                + n_back_ticks;
            if self.get_byte()? == b'`' {
                // Skip excessive backticks.
                self.inc();
                while let b'`' = self.get_byte()? {
                    self.inc();
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Skip a full math block.
    /// Add `Event`s to mark the start and end of the math block and
    /// surrounding text blocks.
    /// Return `Err(())` if no more bytes to process.
    fn process_delimit(&mut self, inline: bool) -> Result<(), ()> {
        if self.index > 0 {
            self.events.push(Event::TextEnd(self.index));
        }

        let delim = if inline {
            self.inline_delimiter
        } else {
            self.block_delimiter
        };
        self.index += delim.left.len();
        self.events.push(Event::Begin(self.index));

        loop {
            self.index += self.string[self.index..].find(&delim.right).ok_or(())?;

            // Check `\`.
            let mut escaped = false;
            let mut checking = self.index;
            loop {
                checking -= 1;
                if self.bytes.get(checking) == Some(&b'\\') {
                    escaped = !escaped;
                } else {
                    break;
                }
            }
            if !escaped {
                let end_event = if inline {
                    Event::InlineEnd(self.index)
                } else {
                    Event::BlockEnd(self.index)
                };
                self.events.push(end_event);
                self.index += delim.right.len();
                self.events.push(Event::Begin(self.index));
                break;
            } else {
                self.index += delim.right.len();
            }
        }

        Ok(())
    }
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

/// Absolute path of the macro file.
pub fn get_macro_path<P>(root: P, macros_path: &Option<String>) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    macros_path
        .as_ref()
        .map(|path| root.as_ref().join(PathBuf::from(path)))
}

/// Extract configuration for katex preprocessor from `book_cfg`.
pub fn get_config(book_cfg: &mdbook::Config) -> Result<KatexConfig, toml::de::Error> {
    let cfg = match book_cfg.get("preprocessor.katex") {
        Some(raw) => raw.clone().try_into(),
        None => Ok(KatexConfig::default()),
    };
    cfg.or_else(|_| Ok(KatexConfig::default()))
}

/// Read file at `path`.
pub fn load_as_string(path: &Path) -> String {
    let display = path.display();

    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {display}: {why}"),
        Ok(file) => file,
    };

    let mut string = String::new();
    if let Err(why) = file.read_to_string(&mut string) {
        panic!("couldn't read {display}: {why}")
    };
    string
}

#[cfg(test)]
mod tests;
