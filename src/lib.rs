use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::vec::Vec;

use katex::Opts;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::renderer::{RenderContext, Renderer};
use mdbook::utils::fs::path_to_root;
use tokio::spawn;
use tokio::task::JoinHandle;

const BLOCK_DELIM: &str = "$$";
const INLINE_DELIM: &str = "$";

#[derive(Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct KatexConfig {
    // options for the katex-rust crate
    pub leqno: bool,
    pub fleqn: bool,
    pub throw_on_error: bool,
    pub error_color: String,
    pub min_rule_thickness: f64,
    pub max_size: f64,
    pub max_expand: i32,
    pub trust: bool,
    // other options
    pub static_css: bool,
    pub include_src: bool,
    pub macros: Option<String>,
}

impl Default for KatexConfig {
    fn default() -> KatexConfig {
        KatexConfig {
            // default options for the katex-rust crate
            // uses defaults specified in: https://katex.org/docs/options.html
            leqno: false,
            fleqn: false,
            throw_on_error: true,
            error_color: String::from("#cc0000"),
            min_rule_thickness: -1.0,
            max_size: f64::INFINITY,
            max_expand: 1000,
            trust: false,
            // other options
            static_css: false,
            include_src: false,
            macros: None,
        }
    }
}

// ensures that both the preprocessor and renderers are enabled
// in the `book.toml`; the renderer forces mdbook to separate all
// renderers into their respective directories, ensuring that the
// html renderer will always be at `{out_dir}/html`
fn enforce_config(cfg: &mdbook::Config) {
    if cfg.get("preprocessor.katex").is_none() {
        panic!("Missing `[preprocessor.katex]` directive in `book.toml`!");
    }
    if cfg.get("output.katex").is_none() {
        panic!("Missing `[output.katex]` directive in `book.toml`!");
    }
    if cfg.get("output.html").is_none() {
        panic!("The katex preprocessor is only compatible with the html renderer!");
    }
}

pub struct KatexProcessor;

// dummy renderer to ensure rendered output is always located
// in the `book/html/` directory
impl Renderer for KatexProcessor {
    fn name(&self) -> &str {
        "katex"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        enforce_config(&ctx.config);
        Ok(())
    }
}

// preprocessor to inject rendered katex blocks and stylesheet
impl Preprocessor for KatexProcessor {
    fn name(&self) -> &str {
        "katex"
    }

    #[tokio::main]
    async fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // enforce config requirements
        enforce_config(&ctx.config);
        // parse TOML config
        let cfg = get_config(&ctx.config)?;
        let (inline_opts, display_opts) = build_opts(ctx, &cfg);
        // get stylesheet header
        let (stylesheet_header, maybe_download_task) =
            katex_header(&ctx.root, &ctx.config.build.build_dir, &cfg).await?;
        let mut paths_w_raw_contents = Vec::new();
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Some(ref path) = chapter.path {
                    if cfg.static_css {
                        paths_w_raw_contents.push((Some(path.clone()), chapter.content.clone()))
                    } else {
                        paths_w_raw_contents.push((None, chapter.content.clone()));
                    }
                }
            }
        });
        let mut tasks = Vec::with_capacity(paths_w_raw_contents.len());
        for (path, content) in paths_w_raw_contents {
            let header = if cfg.static_css {
                format!(
                    "<link rel=\"stylesheet\" href=\"{}katex/katex.min.css\">\n\n",
                    path_to_root(path.unwrap()), // must be `Some` since `static_css`
                )
            } else {
                stylesheet_header.clone()
            };
            tasks.push(spawn(process_chapter(
                content,
                inline_opts.clone(),
                display_opts.clone(),
                header,
                cfg.include_src,
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
        if let Some(download_task) = maybe_download_task {
            download_task.await??;
        }
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "markdown"
    }
}

fn build_opts(ctx: &PreprocessorContext, cfg: &KatexConfig) -> (katex::Opts, katex::Opts) {
    let configure_katex_opts = || -> katex::OptsBuilder {
        katex::Opts::builder()
            .leqno(cfg.leqno)
            .fleqn(cfg.fleqn)
            .throw_on_error(cfg.throw_on_error)
            .error_color(cfg.error_color.clone())
            .min_rule_thickness(cfg.min_rule_thickness)
            .max_size(cfg.max_size)
            .max_expand(cfg.max_expand)
            .trust(cfg.trust)
            .clone()
    };
    // load macros as a HashMap
    let macros = load_macros(ctx, &cfg.macros);
    // inline rendering options
    let inline_opts = configure_katex_opts()
        .display_mode(false)
        .output_type(katex::OutputType::Html)
        .macros(macros.clone())
        .build()
        .unwrap();
    // display rendering options
    let display_opts = configure_katex_opts()
        .display_mode(true)
        .output_type(katex::OutputType::Html)
        .macros(macros)
        .build()
        .unwrap();
    (inline_opts, display_opts)
}

fn load_macros(ctx: &PreprocessorContext, macros_path: &Option<String>) -> HashMap<String, String> {
    // load macros as a HashMap
    let mut map = HashMap::new();
    if let Some(path) = get_macro_path(&ctx.root, macros_path) {
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

/// Render Katex equations in a `Chapter` as HTML, and add the Katex CSS.
async fn process_chapter(
    raw_content: String,
    inline_opts: Opts,
    display_opts: Opts,
    stylesheet_header: String,
    include_src: bool,
) -> String {
    let mut rendered_vec = Vec::new();
    rendered_vec.push(stylesheet_header.to_owned());

    let mut scan = Scan::new(&raw_content);
    scan.run();

    let mut checkpoint = 0;
    let events = scan.events.iter();
    for event in events {
        match *event {
            Event::Begin(begin) => checkpoint = begin,
            Event::TextEnd(end) => {
                let text_block = (&raw_content[checkpoint..end]).into();
                dbg!(&text_block);
                rendered_vec.push(text_block);
            }
            Event::InlineEnd(end) => {
                let inline_feed = (&raw_content[checkpoint..end]).into();
                dbg!(&inline_feed);
                let inline_block = render(inline_feed, inline_opts.clone(), include_src).await;
                dbg!(&inline_block);
                rendered_vec.push(inline_block);
                checkpoint = end;
            }
            Event::BlockEnd(end) => {
                let block_feed = (&raw_content[checkpoint..end]).into();
                dbg!(&block_feed);
                let block = render(block_feed, display_opts.clone(), include_src).await;
                dbg!(&block);
                rendered_vec.push(block);
                checkpoint = end;
            }
        }
    }

    if raw_content.len() - 1 > checkpoint {
        let text_block = (&raw_content[checkpoint..raw_content.len()]).into();
        dbg!(&text_block);
        rendered_vec.push(text_block);
    }
    rendered_vec.join("")
}

#[derive(Debug)]
pub enum Event {
    Begin(usize),
    TextEnd(usize),
    InlineEnd(usize),
    BlockEnd(usize),
}

#[derive(Debug)]
pub struct Scan<'a> {
    string: &'a str,
    bytes: &'a [u8],
    index: usize,
    pub events: Vec<Event>,
}

impl<'a> Scan<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            string,
            bytes: string.as_bytes(),
            index: 0,
            events: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        _ = self.process_byte();
    }

    fn get_byte(&self) -> Result<u8, ()> {
        self.bytes.get(self.index).map(|b| b.to_owned()).ok_or(())
    }

    fn inc(&mut self) {
        self.index += 1;
    }

    fn process_byte(&mut self) -> Result<(), ()> {
        let byte = self.get_byte()?;
        self.inc();
        match byte {
            b'$' => {
                if self.index > 0 {
                    self.events.push(Event::TextEnd(self.index - 1));
                }
                if self.get_byte()? == b'$' {
                    self.inc();
                    self.process_block()
                } else {
                    self.process_inline()
                }
            }
            b'\\' => {
                self.inc();
                self.process_byte()
            }
            b'`' => self.process_backtick(),
            _ => self.process_byte(),
        }
    }

    fn process_backtick(&mut self) -> Result<(), ()> {
        let mut n_back_ticks = 1;
        loop {
            let byte = self.get_byte()?;
            self.inc();
            if byte == b'`' {
                n_back_ticks += 1;
            } else {
                break;
            }
        }
        loop {
            let index = self.index
                + n_back_ticks
                + self.string[self.index..]
                    .find(&"`".repeat(n_back_ticks))
                    .ok_or(())?;
            self.index = index;
            match self.bytes.get(index) {
                Some(byte) if *byte == b'`' => (),
                _ => break,
            }
        }
        self.process_byte()
    }

    fn process_block(&mut self) -> Result<(), ()> {
        self.events.push(Event::Begin(self.index));
        loop {
            let index = self.index + self.string[self.index..].find(BLOCK_DELIM).ok_or(())?;

            // Check `\`.
            let mut escaped = false;
            let mut checking = index;
            loop {
                checking -= 1;
                if self.bytes.get(checking) == Some(&b'\\') {
                    escaped = !escaped;
                } else {
                    break;
                }
            }
            if !escaped {
                self.events.push(Event::BlockEnd(index));
                self.index = index + BLOCK_DELIM.len();
                self.events.push(Event::Begin(self.index));
                break;
            }
        }

        self.process_byte()
    }

    fn process_inline(&mut self) -> Result<(), ()> {
        self.events.push(Event::Begin(self.index));
        loop {
            let index = self.index + self.string[self.index..].find(INLINE_DELIM).ok_or(())?;

            // Check `\`.
            let mut escaped = false;
            let mut checking = index;
            loop {
                checking -= 1;
                if self.bytes.get(checking) == Some(&b'\\') {
                    escaped = !escaped;
                } else {
                    break;
                }
            }
            if !escaped {
                self.events.push(Event::InlineEnd(index));
                self.index = index + INLINE_DELIM.len();
                self.events.push(Event::Begin(self.index));
                break;
            }
        }

        self.process_byte()
    }
}

pub async fn render(item: String, opts: Opts, include_src: bool) -> String {
    let mut rendered_content = String::new();

    // try to render equation
    if let Ok(rendered) = katex::render_with_opts(&item, opts) {
        let rendered = rendered.replace('\n', " ");
        if include_src {
            // Wrap around with `data.katex-src` tag.
            rendered_content.push_str(r#"<data class="katex-src" value=""#);
            rendered_content.push_str(&item.replace('"', r#"\""#));
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

pub fn get_macro_path(root: &Path, macros_path: &Option<String>) -> Option<PathBuf> {
    macros_path
        .as_ref()
        .map(|path| root.join(PathBuf::from(path)))
}

pub fn get_config(book_cfg: &mdbook::Config) -> Result<KatexConfig, toml::de::Error> {
    let cfg = match book_cfg.get("preprocessor.katex") {
        Some(raw) => raw.clone().try_into(),
        None => Ok(KatexConfig::default()),
    };
    cfg.or_else(|_| Ok(KatexConfig::default()))
}

pub fn load_as_string(path: &Path) -> String {
    let display = path.display();

    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut string = String::new();
    if let Err(why) = file.read_to_string(&mut string) {
        panic!("couldn't read {}: {}", display, why)
    };
    string
}

type SideEffectHandle = JoinHandle<Result<(), Error>>;
async fn katex_header(
    build_root: &Path,
    build_dir: &Path,
    cfg: &KatexConfig,
) -> Result<(String, Option<SideEffectHandle>), Error> {
    // constants
    let cdn_root = "https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/";
    let stylesheet_url = format!("{}katex.min.css", cdn_root);
    let integrity = "sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X";

    if cfg.static_css {
        Ok((
            "".to_owned(), // not used
            Some(spawn(download_static_css(
                build_root.into(),
                build_dir.into(),
                stylesheet_url,
                cdn_root.into(),
            ))),
        ))
    } else {
        Ok((format!(
                "<link rel=\"stylesheet\" href=\"{}\" integrity=\"{}\" crossorigin=\"anonymous\">\n\n",
                stylesheet_url,
                integrity,
            ), None))
    }
}

async fn download_static_css(
    build_root: PathBuf,
    build_dir: PathBuf,
    stylesheet_url: String,
    cdn_root: String,
) -> Result<(), Error> {
    // create katex resource directory
    let mut katex_dir_path = build_root.join(build_dir);
    katex_dir_path.push("html/katex");
    if !katex_dir_path.exists() {
        std::fs::create_dir_all(katex_dir_path.as_path())?;
    }

    // download or fetch stylesheet content
    let mut stylesheet_path = katex_dir_path.clone();
    stylesheet_path.push("katex.min.css");

    let mut stylesheet: String;
    if !stylesheet_path.exists() {
        // download stylesheet content
        let stylesheet_response = reqwest::get(stylesheet_url).await?;
        stylesheet = String::from(std::str::from_utf8(&stylesheet_response.bytes().await?)?);
        // create stylesheet file and populate it with the content
        let mut stylesheet_file = File::create(stylesheet_path.as_path())?;
        stylesheet_file.write_all(stylesheet.as_str().as_bytes())?;
    } else {
        // read stylesheet content from disk
        stylesheet = String::new();
        let mut stylesheet_file = File::open(stylesheet_path.as_path())?;
        stylesheet_file.read_to_string(&mut stylesheet)?;
    }

    // download all resources from stylesheet
    let url_pattern = Regex::new(r"(url)\s*[(]([^()]*)[)]").unwrap();
    let rel_pattern = Regex::new(r"[.][.][/\\]|[.][/\\]").unwrap();
    let mut resources: HashSet<String> = HashSet::new();
    let mut tasks = Vec::new();
    for capture in url_pattern.captures_iter(&stylesheet) {
        let resource_name = String::from(&capture[2]);
        // sanitize resource path
        let mut resource_path = katex_dir_path.clone();
        resource_path.push(&resource_name);
        resource_path = PathBuf::from(String::from(
            rel_pattern.replace_all(resource_path.to_str().unwrap(), ""),
        ));
        // create resource path and populate content
        if !resource_path.as_path().exists() {
            // don't download resources if they already exist
            if resources.insert(String::from(&capture[2])) {
                tasks.push(spawn(download_static_fonts(
                    resource_path,
                    cdn_root.to_owned(),
                    resource_name,
                )));
            }
        }
    }
    for task in tasks {
        task.await??;
    }
    Ok(())
}

async fn download_static_fonts(
    resource_path: PathBuf,
    cdn_root: String,
    resource_name: String,
) -> Result<(), Error> {
    // create all leading directories
    let mut resource_parent_dir = resource_path.clone();
    resource_parent_dir.pop();
    std::fs::create_dir_all(resource_parent_dir.as_path())?;
    // create resource file
    let mut resource_file = File::create(resource_path)?;
    // download content
    let resource_url = format!("{}{}", cdn_root, &resource_name);
    let resource_response = reqwest::get(resource_url).await?;
    // populate file with content
    resource_file.write_all(&resource_response.bytes().await?)?;
    Ok(())
}

#[cfg(test)]
mod tests;
