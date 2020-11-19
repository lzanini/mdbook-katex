extern crate katex;
extern crate toml;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;
use std::process;

pub fn make_app() -> App<'static, 'static> {
    App::new("mdbook-katex")
        .about("A preprocessor that converts KaTex equations to HTML.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let preprocessor = KatexProcessor::new();
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    }
    let result = handle_preprocessing(&preprocessor);
    if let Err(e) = result {
        eprintln!("{}", e);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

struct KatexProcessor;

impl KatexProcessor {
    fn new() -> Self {
        Self
    }

    // Take as input the content of a Chapter, and returns a String corresponding to the new content.
    fn process(&self, content: &str, macros_path: &Option<String>) -> String {
        let macros = self.load_macros(macros_path);
        self.render(&content, macros)
    }

    fn load_macros(&self, macros_path: &Option<String>) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(path) = macros_path {
            let macro_str = load_as_string(&path);
            for couple in macro_str.split("\n") {
                if let Some('\\') = couple.chars().next() {
                    let couple: Vec<&str> = couple.split(":").collect();
                    map.insert(String::from(couple[0]), String::from(couple[1]));
                }
            }
        }
        map
    }

    fn render(&self, content: &str, macros: HashMap<String, String>) -> String {
        // add katex css cdn
        let header = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css" integrity="sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X" crossorigin="anonymous">"#;
        let mut html = String::from(header);
        html.push_str("\n\n");
        // render equations
        let content = self.render_separator(&content, "$$", true, macros.clone());
        // render inline
        let content = self.render_separator(&content, "$", false, macros.clone());
        // add rendered md content
        html.push_str(&content);
        html
    }

    // split string according to some separator <sep>, but ignore blackslashed \<sep>
    fn split_with_escape<'a>(&self, string: &'a str, separator: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut splits = string.split(separator);
        let mut current_split = splits.next();
        while let Some(split) = current_split {
            let mut result_split = String::from(split);
            while let Some('\\') = current_split.unwrap().chars().last() {
                result_split.pop();
                result_split.push_str("$");
                current_split = splits.next();
                if let Some(split) = current_split {
                    result_split.push_str(split);
                }
            }
            result.push(result_split);
            current_split = splits.next()
        }
        result
    }

    fn render_separator(
        &self,
        string: &str,
        separator: &str,
        display: bool,
        macros: HashMap<String, String>,
    ) -> String {
        let mut html = String::new();
        let mut k = 0;
        for item in self.split_with_escape(&string, &separator) {
            if k % 2 == 1 {
                let ops = katex::Opts::builder()
                    .display_mode(display)
                    .output_type(katex::OutputType::Html)
                    .macros(macros.clone())
                    .build()
                    .unwrap();
                let result = katex::render_with_opts(&item, ops);
                if let Ok(rendered) = result {
                    html.push_str(&rendered)
                } else {
                    html.push_str(&item)
                }
            } else {
                html.push_str(&item)
            }
            k += 1;
        }
        html
    }
}

impl Preprocessor for KatexProcessor {
    fn name(&self) -> &str {
        "katex"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let mut macros_path = None;
        if let Some(config) = ctx.config.get_preprocessor(KatexProcessor.name()) {
            if let Some(toml::value::Value::String(macros_value)) = config.get("macros") {
                macros_path = Some(String::from(macros_value));
            }
        }
        let mut new_book = book.clone();
        new_book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = self.process(&chapter.content, &macros_path)
            }
        });
        Ok(new_book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn load_as_string(path: &str) -> String {
    let path = Path::new(path);
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
