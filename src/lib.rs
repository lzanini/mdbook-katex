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

use mdbook::utils::fs::path_to_root;
use tokio::spawn;
use tokio::task::JoinHandle;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Delimiter {
    pub left: String,
    pub right: String,
}

impl Delimiter {
    pub fn same(delimiter: String) -> Self {
        Self {
            left: delimiter.clone(),
            right: delimiter,
        }
    }

    pub fn first(&self) -> u8 {
        self.left.as_bytes()[0]
    }

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct KatexConfig {
    // options for the katex-rust crate
    pub output: String,
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
    pub no_css: bool,
    pub include_src: bool,
    pub macros: Option<String>,
    pub block_delimiter: Delimiter,
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
            static_css: false,
            no_css: false,
            include_src: false,
            macros: None,
            block_delimiter: Delimiter::same("$$".into()),
            inline_delimiter: Delimiter::same("$".into()),
        }
    }
}

impl KatexConfig {
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

    pub fn build_opts<P>(&self, root: P) -> (katex::Opts, katex::Opts, ExtraOpts)
    where
        P: AsRef<Path>,
    {
        // load macros as a HashMap
        let macros = load_macros(root, &self.macros);

        self.build_opts_from_macros(macros)
    }

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

#[derive(Clone, Debug)]
pub struct ExtraOpts {
    pub include_src: bool,
    pub block_delimiter: Delimiter,
    pub inline_delimiter: Delimiter,
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

// preprocessor to inject rendered katex blocks and stylesheet
impl Preprocessor for KatexProcessor {
    fn name(&self) -> &str {
        "katex"
    }

    #[tokio::main]
    async fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // parse TOML config
        let cfg = get_config(&ctx.config)?;
        if cfg.static_css {
            // enforce config requirements
            enforce_config(&ctx.config);
        }
        let (inline_opts, display_opts, extra_opts) = cfg.build_opts(&ctx.root);
        // get stylesheet header
        let (stylesheet_header, maybe_download_task) =
            katex_header(&ctx.root, &ctx.config.build.build_dir, &cfg).await?;
        let mut paths_w_raw_contents = Vec::new();
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Some(ref path) = chapter.path {
                    if !cfg.no_css && cfg.static_css {
                        paths_w_raw_contents.push((Some(path.clone()), chapter.content.clone()))
                    } else {
                        paths_w_raw_contents.push((None, chapter.content.clone()));
                    }
                }
            }
        });
        let mut tasks = Vec::with_capacity(paths_w_raw_contents.len());
        for (path, content) in paths_w_raw_contents {
            let header = if cfg.no_css {
                "".into()
            } else if cfg.static_css {
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
        if let Some(download_task) = maybe_download_task {
            download_task.await??;
        }
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "markdown"
    }
}

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

pub enum Render {
    Text(String),
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
    block_delimiter: &'a Delimiter,
    inline_delimiter: &'a Delimiter,
}

impl<'a> Scan<'a> {
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

    pub fn run(&mut self) {
        while let Ok(()) = self.process_byte() {}
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

pub async fn render(item: String, opts: Opts, extra_opts: ExtraOpts) -> String {
    let mut rendered_content = String::new();

    // try to render equation
    if let Ok(rendered) = katex::render_with_opts(&item, opts) {
        let rendered = rendered.replace('\n', " ");
        if extra_opts.include_src {
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

pub fn get_macro_path<P>(root: P, macros_path: &Option<String>) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    macros_path
        .as_ref()
        .map(|path| root.as_ref().join(PathBuf::from(path)))
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
        Err(why) => panic!("couldn't open {display}: {why}"),
        Ok(file) => file,
    };

    let mut string = String::new();
    if let Err(why) = file.read_to_string(&mut string) {
        panic!("couldn't read {display}: {why}")
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
    let stylesheet_url = format!("{cdn_root}katex.min.css");
    let integrity = "sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X";

    if cfg.static_css {
        eprintln!(
            "
[WARNNING] mdbook-katex: `static-css` in `book.toml` is deprecated and will be removed in v0.4.0.
Please use `no-css` instead. See https://github.com/lzanini/mdbook-katex/issues/68"
        );
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
                "<link rel=\"stylesheet\" href=\"{stylesheet_url}\" integrity=\"{integrity}\" crossorigin=\"anonymous\">\n\n",
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
