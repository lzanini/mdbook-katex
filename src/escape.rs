//! Escaping math blocks to fix KaTeX rendering.

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
/// Delimiter also need to be escaped, e.g. `\(,\)` and `\[,\]`.
pub fn escape_math_with_delimiter(item: &str, delimiter: &Delimiter) -> String {
    let mut result = String::new();
    escape_math(&delimiter.left, &mut result);
    escape_math(item, &mut result);
    escape_math(&delimiter.right, &mut result);
    result
}

/// This is a amazing but useful little trick.
/// Mdbook's markdown engine will parse a part of KaTeX formula into HTML, e.g. `$[x^n](f + g)$`.
/// So if we escape the math formula in advance so that it passes through the markdown
/// engine as the original formula, it will be rendered correctly by katex.js.
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
