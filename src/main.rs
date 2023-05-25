use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook_katex::preprocess::KatexProcessor;
use std::io::{self, Read};

/// Parse CLI options.
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

/// Produce a warning on mdBook version mismatch.
fn check_mdbook_version(version: &str) {
    if version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "This mdbook-katex was built against mdbook v{}, \
            but we are being called from mdbook v{version}. \
            If you have any issue, this might be a reason.",
            mdbook::MDBOOK_VERSION,
        )
    }
}

/// Tell mdBook if we support what it asks for.
fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> Result<(), Error> {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);
    if supported {
        Ok(())
    } else {
        Err(Error::msg(format!(
            "The katex preprocessor does not support the '{renderer}' renderer",
        )))
    }
}

/// Preprocess `book` using `pre` and print it out.
fn handle_preprocessing(
    pre: &dyn Preprocessor,
    ctx: &PreprocessorContext,
    book: Book,
) -> Result<(), Error> {
    check_mdbook_version(&ctx.mdbook_version);

    let processed_book = pre.run(ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    // set up app
    let matches = make_app().get_matches();
    let pre = KatexProcessor;

    // determine what behaviour has been requested
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        handle_supports(&pre, sub_args)
    } else if let Ok((ctx, book)) = {
        // grab book data from stdin
        let mut book_data = String::new();
        io::stdin().read_to_string(&mut book_data)?;

        CmdPreprocessor::parse_input(book_data.as_bytes())
    } {
        // handle preprocessing
        handle_preprocessing(&pre, &ctx, book)
    } else {
        Err(Error::msg(
            "mdbook-katex did not recognize the argument passed in.",
        ))
    }
}
