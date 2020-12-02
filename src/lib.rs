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
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let preprocessor = KatexProcessor;
        assert_eq!(preprocessor.name(), "katex")
    }

    #[test]
    fn test_support_html() {
        let preprocessor = KatexProcessor;
        assert!(preprocessor.supports_renderer("html"));
        assert!(!preprocessor.supports_renderer("other_renderer"))
    }

    fn mock_build_opts(macros: HashMap<String, String>) -> (katex::Opts, katex::Opts) {
        let inline_opts = katex::Opts::builder()
            .display_mode(false)
            .output_type(katex::OutputType::Html)
            .macros(macros.clone())
            .build()
            .unwrap();
        let display_opts = katex::Opts::builder()
            .display_mode(true)
            .output_type(katex::OutputType::Html)
            .macros(macros)
            .build()
            .unwrap();
        (inline_opts, display_opts)
    }

    #[test]
    fn test_rendering_without_math() {
        let preprocessor = KatexProcessor;
        let macros = HashMap::new();
        let (inline_opts, display_opts) = mock_build_opts(macros);
        let raw_content = r"Some text, and more text.";
        let mut expected_output = katex_header();
        expected_output.push_str(raw_content);
        let rendered_content =
            preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
        debug_assert_eq!(expected_output, rendered_content);
    }

    #[test]
    fn test_dollar_escaping() {
        let preprocessor = KatexProcessor;
        let macros = HashMap::new();
        let (inline_opts, display_opts) = mock_build_opts(macros);
        let raw_content = r"Some text, \$\$ and more text.";
        let mut expected_output = katex_header();
        expected_output.push_str(r"Some text, $$ and more text.");
        let rendered_content =
            preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
        debug_assert_eq!(expected_output, rendered_content);
    }

    #[test]
    fn test_inline_rendering() {
        let preprocessor = KatexProcessor;
        let macros = HashMap::new();
        let (inline_opts, display_opts) = mock_build_opts(macros);
        let raw_content = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
        let mut expected_output = katex_header();
        expected_output.push_str("Some text, <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord\">∇</span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.68889em;vertical-align:0em;\"></span><span class=\"mord\"><span class=\"mord\"><span class=\"mord mathbb\">R</span></span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.664392em;\"><span style=\"top:-3.063em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">n</span></span></span></span></span></span></span></span></span></span></span>, and more text.");
        let rendered_content =
            preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
        debug_assert_eq!(expected_output, rendered_content);
    }

    #[test]
    fn test_display_rendering() {
        let preprocessor = KatexProcessor;
        let macros = HashMap::new();
        let (inline_opts, display_opts) = mock_build_opts(macros);
        let raw_content = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
        let mut expected_output = katex_header();
        expected_output.push_str("Some text, <span class=\"katex\"><span class=\"katex-html\" aria-hidden=\"true\"><span class=\"base\"><span class=\"strut\" style=\"height:1em;vertical-align:-0.25em;\"></span><span class=\"mord\">∇</span><span class=\"mord mathnormal\" style=\"margin-right:0.10764em;\">f</span><span class=\"mopen\">(</span><span class=\"mord mathnormal\">x</span><span class=\"mclose\">)</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span><span class=\"mrel\">∈</span><span class=\"mspace\" style=\"margin-right:0.2777777777777778em;\"></span></span><span class=\"base\"><span class=\"strut\" style=\"height:0.68889em;vertical-align:0em;\"></span><span class=\"mord\"><span class=\"mord\"><span class=\"mord mathbb\">R</span></span><span class=\"msupsub\"><span class=\"vlist-t\"><span class=\"vlist-r\"><span class=\"vlist\" style=\"height:0.664392em;\"><span style=\"top:-3.063em;margin-right:0.05em;\"><span class=\"pstrut\" style=\"height:2.7em;\"></span><span class=\"sizing reset-size6 size3 mtight\"><span class=\"mord mathnormal mtight\">n</span></span></span></span></span></span></span></span></span></span></span>, and more text.");
        let rendered_content =
            preprocessor.process_chapter(&raw_content, &inline_opts, &display_opts);
        debug_assert_eq!(expected_output, rendered_content);
    }

    #[test]
    fn test_macros_without_argument() {
        let preprocessor = KatexProcessor;
        let mut macros = HashMap::new();
        macros.insert(String::from(r"\grad"), String::from(r"\nabla"));
        let (inline_opts, display_opts) = mock_build_opts(macros);
        let raw_content_no_macro = r"Some text, $\nabla f(x) \in \mathbb{R}^n$, and more text.";
        let raw_content_macro = r"Some text, $\grad f(x) \in \mathbb{R}^n$, and more text.";
        let rendered_content_macro =
            preprocessor.process_chapter(&raw_content_macro, &inline_opts, &display_opts);
        let rendered_content_no_macro =
            preprocessor.process_chapter(&raw_content_no_macro, &inline_opts, &display_opts);
        debug_assert_eq!(rendered_content_macro, rendered_content_no_macro);
    }

    #[test]
    fn test_macros_with_argument() {
        let preprocessor = KatexProcessor;
        let mut macros = HashMap::new();
        macros.insert(String::from(r"\R"), String::from(r"\mathbb{R}^#1"));
        let (inline_opts, display_opts) = mock_build_opts(macros);
        let raw_content_no_macro = r"Some text, $\nabla f(x) \in \mathbb{R}^1$, and more text.";
        let raw_content_macro = r"Some text, $\nabla f(x) \in \R{1}$, and more text.";
        let rendered_content_macro =
            preprocessor.process_chapter(&raw_content_macro, &inline_opts, &display_opts);
        let rendered_content_no_macro =
            preprocessor.process_chapter(&raw_content_no_macro, &inline_opts, &display_opts);
        debug_assert_eq!(rendered_content_macro, rendered_content_no_macro);
    }
}
