//! Preprocess math blocks using KaTeX for mdBook.

use crate::scan::Delimiter;

/// A render job for `process_chapter`.
pub enum Render<'a> {
    /// No need to render.
    Text(&'a str),
    /// A render task for a math inline block.
    InlineTask(&'a str),
    /// A render task for a math display block.
    DisplayTask(&'a str),
}

/// Escape a math block `item` into a delimited string.
pub fn escape_math_with_delimiter(item: &str, delimiter: &Delimiter) -> String {
    let mut result = String::new();
    escape_math(&delimiter.left, &mut result);
    escape_math(item, &mut result);
    escape_math(&delimiter.right, &mut result);
    result
}

/// Escape the math block `item` to `result`.
pub fn escape_math(item: &str, result: &mut String) {
    for c in item.chars() {
        match c {
            '_' => {
                result.push_str("\\_");
            }
            '*' => {
                result.push_str("\\*");
            }
            '\\' => {
                result.push_str("\\\\");
            }
            _ => {
                result.push(c);
            }
        }
    }
}
