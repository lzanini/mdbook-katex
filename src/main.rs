use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook_katex::KatexProcessor;
use std::io::{self, Read};

pub fn make_app() -> Command {
    Command::new("mdbook-katex")
        .version(crate_version!())
        .about("A preprocessor that renders KaTex equations to HTML.")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn check_mdbook_version(version: &String) {
    if version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "This mdbook-katex was built against mdbook v{}, \
            but we are being called from mdbook v{}. \
            If you have any issue, this might be a reason.",
            mdbook::MDBOOK_VERSION,
            &version
        )
    }
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> Result<(), Error> {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);
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
    check_mdbook_version(&ctx.mdbook_version);

    let processed_book = pre.run(ctx, book.clone())?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    // set up app
    let matches = make_app().get_matches();
    let pre = KatexProcessor;

    // grab book data from stdin
    let mut book_data = String::new();
    io::stdin().read_to_string(&mut book_data)?;

    // determine what behaviour has been requested
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        return handle_supports(&pre, sub_args);
    } else if let Ok((ctx, book)) = CmdPreprocessor::parse_input(book_data.as_bytes()) {
        // handle preprocessing
        return handle_preprocessing(&pre, &ctx, &book);
    }

    Err(Error::msg(
        "mdbook-katex did not recognize the argument passed in.",
    ))
}
