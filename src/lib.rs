#![deny(missing_docs)]
//! Preprocess math blocks using KaTeX for mdBook.
use std::{
    borrow::Cow,
    collections::HashMap,
    collections::VecDeque,
    fs::File,
    io::{stderr, Read},
    path::{Path, PathBuf},
};

use mdbook_preprocessor::{book::Book, errors::Result, Preprocessor, PreprocessorContext};
use rayon::iter::*;
use serde_derive::{Deserialize, Serialize};
use tracing::*;
use tracing_subscriber::EnvFilter;

use {
    cfg::*,
    escape::*,
    preprocess::*,
    scan::{Event, *},
};

pub mod cfg;
pub mod escape;
pub mod preprocess;
pub mod scan;

#[cfg(feature = "pre-render")]
pub mod render;

#[cfg(feature = "pre-render")]
pub use render::*;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub fn init_tracing() {
    _ = tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_ansi(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .try_init();
}
