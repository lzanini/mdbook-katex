use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub struct KatexProcessor;

impl Preprocessor for KatexProcessor {
    fn name(&self) -> &str {
        "katex"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let (inline_opts, display_opts) = self.build_opts(ctx);
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content =
                    self.process_chapter(&chapter.content, &inline_opts, &display_opts)
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

impl KatexProcessor {
    fn build_opts(&self, ctx: &PreprocessorContext) -> (katex::Opts, katex::Opts) {
        // load macros as a HashMap
        let macros = Self::load_macros(ctx);
        // inline rendering options
        let inline_opts = katex::Opts::builder()
            .display_mode(false)
            .output_type(katex::OutputType::Html)
            .macros(macros.clone())
            .build()
            .unwrap();
        // display rendering options
        let display_opts = katex::Opts::builder()
            .display_mode(true)
            .output_type(katex::OutputType::Html)
            .macros(macros)
            .build()
            .unwrap();
        (inline_opts, display_opts)
    }

    fn load_macros(ctx: &PreprocessorContext) -> HashMap<String, String> {
        // get macros path from context
        let mut macros_path = None;
        if let Some(config) = ctx.config.get_preprocessor("katex") {
            if let Some(toml::value::Value::String(macros_value)) = config.get("macros") {
                macros_path = Some(Path::new(macros_value));
            }
        }
        // load macros as a HashMap
        let mut map = HashMap::new();
        if let Some(path) = macros_path {
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
    ) -> String {
        // add katex css
        let mut rendered_content = katex_header();
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
                    rendered_content.push_str(&rendered)
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

fn katex_header() -> String {
    String::from("<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css\" integrity=\"sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X\" crossorigin=\"anonymous\">\n\n")
}

#[cfg(test)]
mod tests;
