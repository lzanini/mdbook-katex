#![deny(missing_docs)]
//! Preprocess math blocks using KaTeX for mdBook.
pub mod cfg;
pub mod escape;
pub mod preprocess;
pub mod scan;

#[cfg(feature = "pre-render")]
pub mod render;

#[cfg(feature = "pre-render")]
pub mod preprocess_render;

#[cfg(feature = "pre-render")]
pub mod cfg_render;

#[cfg(test)]
mod tests;
