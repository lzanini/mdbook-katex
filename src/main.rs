extern crate katex;
extern crate toml;

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::renderer::{RenderContext, Renderer};
use mdbook_katex2::KatexProcessor;
use std::io::{self, Read};

pub fn make_app() -> App<'static> {
    App::new("mdbook-katex")
        .about("A preprocessor that renders KaTex equations to HTML.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn check_mdbook_version(version: &String) -> Result<(), Error> {
    if version != mdbook::MDBOOK_VERSION {
        Err(Error::msg(format!(
            "Katex preprocessor/renderer using different mdbook version, {},\
            than it was built against, {}",
            mdbook::MDBOOK_VERSION,
            &version
        )))
    } else {
        Ok(())
    }
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> Result<(), Error> {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);
    if supported {
        Ok(())
    } else {
        Err(Error::msg(format!(
            "The katex preprocessor does not support the '{}' renderer",
            &renderer
        )))
    }
}

fn handle_preprocessing(
    pre: &dyn Preprocessor,
    ctx: &PreprocessorContext,
    book: &Book,
) -> Result<(), Error> {
    // check mdbook version
    check_mdbook_version(&ctx.mdbook_version)?;

    let processed_book = pre.run(&ctx, book.clone())?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn handle_rendering(ctx: &RenderContext, rend: &dyn Renderer) -> Result<(), Error> {
    check_mdbook_version(&ctx.version)?;
    rend.render(&ctx)
}

fn main() -> Result<(), Error> {
    // grab book data from stdin
    let mut book_data = String::new();
    io::stdin().read_to_string(&mut book_data)?;

    // set up app
    let matches = make_app().get_matches();
    let pre = KatexProcessor;

    // determine what behaviour has been requested
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        return handle_supports(&pre, &sub_args);
    } else if let Ok((ctx, book)) = CmdPreprocessor::parse_input(book_data.as_bytes()) {
        // handle preprocessing
        return handle_preprocessing(&pre, &ctx, &book);
    } else if let Ok(ctx) = RenderContext::from_json(book_data.as_bytes()) {
        // handle rendering
        return handle_rendering(&ctx, &pre);
    }

    Err(Error::msg(
        "The katex preprocessor/renderer did not understand what you wanted\
        to do",
    ))
}
