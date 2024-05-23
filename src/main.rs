use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_katex::{init_tracing, preprocess::KatexProcessor};
use std::io;
use tracing::*;

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
        warn!(
            "This mdbook-katex was built against mdbook v{}, \
            but we are being called from mdbook v{version}. \
            If you have any issue, this might be a reason.",
            mdbook::MDBOOK_VERSION,
        )
    }
}

/// Tell mdBook if we support what it asks for.
fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> Result<()> {
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
fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    check_mdbook_version(&ctx.mdbook_version);

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn main() -> Result<()> {
    init_tracing();

    // set up app
    let matches = make_app().get_matches();
    let pre = KatexProcessor;

    // determine what behaviour has been requested
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        handle_supports(&pre, sub_args)
    } else {
        // handle preprocessing
        handle_preprocessing(&pre)
    }
}
