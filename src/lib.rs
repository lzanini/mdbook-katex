#![deny(missing_docs)]
//! Preprocess math blocks using KaTeX for mdBook.
pub mod cfg;
pub mod preprocess;
pub mod render;
pub mod scan;

#[cfg(test)]
mod tests;
