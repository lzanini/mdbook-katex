use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::vec::Vec;

use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use toml;

use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::renderer::{RenderContext, Renderer};
use mdbook::utils::fs::path_to_root;

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

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // enforce config requirements
        enforce_config(&ctx.config);
        // parse TOML config
        let cfg = get_config(&ctx.config)?;
        let (inline_opts, display_opts) = self.build_opts(&ctx, &cfg);
        // get stylesheet header
        let stylesheet_header_generator =
            katex_header(&ctx.root, &ctx.config.build.build_dir, &cfg)?;
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                let stylesheet_header =
                    stylesheet_header_generator(path_to_root(chapter.path.clone().unwrap()));
                chapter.content = self.process_chapter(
                    &chapter.content,
                    &inline_opts,
                    &display_opts,
                    &stylesheet_header,
                )
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "katex"
    }
}

impl KatexProcessor {
    fn build_opts(
        &self,
        ctx: &PreprocessorContext,
        cfg: &KatexConfig,
    ) -> (katex::Opts, katex::Opts) {
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
        let macros = Self::load_macros(&ctx, &cfg.macros);
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

    fn load_macros(
        ctx: &PreprocessorContext,
        macros_path: &Option<String>,
    ) -> HashMap<String, String> {
        // load macros as a HashMap
        let mut map = HashMap::new();
        if let Some(path) = get_macro_path(&ctx.root, &macros_path) {
            let macro_str = load_as_string(&path);
            for couple in macro_str.split("\n") {
                // only consider lines starting with a backslash
                if let Some('\\') = couple.chars().next() {
                    let couple: Vec<&str> = couple.splitn(2, ":").collect();
                    map.insert(String::from(couple[0]), String::from(couple[1]));
                }
            }
        }
        map
    }

    // render Katex equations in HTML, and add the Katex CSS
    fn process_chapter(
        &self,
        raw_content: &str,
        inline_opts: &katex::Opts,
        display_opts: &katex::Opts,
        stylesheet_header: &String,
    ) -> String {
        let mut rendered_content = stylesheet_header.clone();
        // render display equations
        let content = Self::render_between_delimiters(&raw_content, "$$", display_opts, false);
        // render inline equations
        let content = Self::render_between_delimiters(&content, "$", inline_opts, true);
        rendered_content.push_str(&content);
        rendered_content
    }

    // render equations between given delimiters, with specified options
    fn render_between_delimiters(
        raw_content: &str,
        delimiters: &str,
        opts: &katex::Opts,
        escape_backslash: bool,
    ) -> String {
        let mut rendered_content = String::new();
        let mut inside_delimiters = false;
        for item in Self::split(&raw_content, &delimiters, escape_backslash) {
            if inside_delimiters {
                // try to render equation
                if let Ok(rendered) = katex::render_with_opts(&item, opts) {
                    rendered_content.push_str(&rendered.replace("\n", " "))
                // if rendering fails, keep the unrendered equation
                } else {
                    rendered_content.push_str(&item)
                }
            // outside delimiters
            } else {
                rendered_content.push_str(&item)
            }
            inside_delimiters = !inside_delimiters;
        }
        rendered_content
    }

    fn split(string: &str, separator: &str, escape_backslash: bool) -> Vec<String> {
        let mut result = Vec::new();
        let mut splits = string.split(separator);
        let mut current_split = splits.next();
        // iterate over splits
        while let Some(substring) = current_split {
            let mut result_split = String::from(substring);
            if escape_backslash {
                // while the current split ends with a backslash
                while let Some('\\') = current_split.unwrap().chars().last() {
                    // removes the backslash, add the separator back, and add the next split
                    result_split.pop();
                    result_split.push_str(separator);
                    current_split = splits.next();
                    if let Some(split) = current_split {
                        result_split.push_str(split);
                    }
                }
            }
            result.push(result_split);
            current_split = splits.next()
        }
        result
    }
}

pub fn get_macro_path(root: &PathBuf, macros_path: &Option<String>) -> Option<PathBuf> {
    match macros_path {
        Some(path) => Some(root.join(PathBuf::from(path))),
        _ => None,
    }
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

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => (),
    };
    string
}

fn katex_header(
    build_root: &PathBuf,
    build_dir: &PathBuf,
    cfg: &KatexConfig,
) -> Result<Box<dyn Fn(String) -> String>, Error> {
    // constants
    let cdn_root = "https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/";
    let stylesheet_url = format!("{}katex.min.css", cdn_root);
    let integrity = "sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X";

    if cfg.static_css {
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
            let stylesheet_response = reqwest::blocking::get(stylesheet_url)?;
            stylesheet = String::from(std::str::from_utf8(&stylesheet_response.bytes()?)?);
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
        lazy_static! {
            static ref URL_PATTERN: Regex = Regex::new(r"(url)\s*[(]([^()]*)[)]").unwrap();
            static ref REL_PATTERN: Regex = Regex::new(r"[.][.][/\\]|[.][/\\]").unwrap();
        }
        let mut resources: HashSet<String> = HashSet::new();
        for capture in URL_PATTERN.captures_iter(&stylesheet) {
            let resource_name = String::from(&capture[2]);
            // sanitize resource path
            let mut resource_path = katex_dir_path.clone();
            resource_path.push(&resource_name);
            resource_path = PathBuf::from(String::from(
                REL_PATTERN.replace_all(resource_path.to_str().unwrap(), ""),
            ));
            // create resource path and populate content
            if !resource_path.as_path().exists() {
                // don't download resources if they already exist
                if resources.insert(String::from(&capture[2])) {
                    // create all leading directories
                    let mut resource_parent_dir = resource_path.clone();
                    resource_parent_dir.pop();
                    std::fs::create_dir_all(resource_parent_dir.as_path())?;
                    // create resource file
                    let mut resource_file = File::create(resource_path)?;
                    // download content
                    let resource_url = format!("{}{}", cdn_root, &resource_name);
                    let resource_response = reqwest::blocking::get(&resource_url)?;
                    // populate file with content
                    resource_file.write_all(&resource_response.bytes()?)?;
                }
            }
        }

        // return closure capable of generating relative paths to the katex
        // resources
        Ok(Box::new(move |path: String| -> String {
            // generate a style element with a relative local path to
            // the katex stylesheet
            String::from(format!(
                "<link rel=\"stylesheet\" href=\"{}katex/katex.min.css\">\n\n",
                path,
            ))
        }))
    } else {
        let stylesheet = String::from(format!(
            "<link rel=\"stylesheet\" href=\"{}\" integrity=\"{}\" crossorigin=\"anonymous\">\n\n",
            stylesheet_url, integrity,
        ));
        Ok(Box::new(move |_: String| -> String { stylesheet.clone() }))
    }
}

#[cfg(test)]
mod tests;
