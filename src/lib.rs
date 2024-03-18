#![deny(missing_docs)]
//! Preprocess math blocks using KaTeX for mdBook.
use std::{
    borrow::Cow,
    collections::HashMap,
    collections::VecDeque,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use mdbook::{
    book::Book,
    errors::Result,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use rayon::iter::*;
use serde_derive::{Deserialize, Serialize};

use {cfg::*, escape::*, preprocess::*, render::*, scan::*};

pub mod cfg;
pub mod escape;
pub mod preprocess;
pub mod scan;

#[cfg(feature = "pre-render")]
pub mod render;

#[cfg(test)]
mod tests;
