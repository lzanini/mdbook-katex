//! Configurations for preprocessing KaTeX.
use super::*;

/// Configuration for KaTeX preprocessor,
/// including options for `katex-rs` and feature options.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct KatexConfig {
    // options for the katex-rust crate
    /// KaTeX output type.
    pub output: String,
    /// Whether to have `\tags` rendered on the left instead of the right.
    pub leqno: bool,
    /// Whether to make display math flush left.
    pub fleqn: bool,
    /// Whether to let KaTeX throw a ParseError for invalid LaTeX.
    pub throw_on_error: bool,
    /// Color used for invalid LaTeX.
    pub error_color: String,
    /// Specifies a minimum thickness, in ems.
    pub min_rule_thickness: f64,
    /// Max size for user-specified sizes.
    pub max_size: f64,
    /// Limit the number of macro expansions to the specified number.
    pub max_expand: i32,
    /// Whether to trust users' input.
    pub trust: bool,
    // other options
    /// Do not inject KaTeX CSS headers.
    pub no_css: bool,
    /// Include math source in rendered HTML.
    pub include_src: bool,
    /// Path to macro file.
    pub macros: Option<String>,
    /// Delimiter for math display block.
    pub block_delimiter: Delimiter,
    /// Delimiter for math inline block.
    pub inline_delimiter: Delimiter,
    /// Use katex.rs to pre-render math equations.
    pub pre_render: bool,
}

impl Default for KatexConfig {
    fn default() -> KatexConfig {
        KatexConfig {
            // default options for the katex-rust crate
            // uses defaults specified in: https://katex.org/docs/options.html
            output: "html".into(),
            leqno: false,
            fleqn: false,
            throw_on_error: true,
            error_color: String::from("#cc0000"),
            min_rule_thickness: -1.0,
            max_size: f64::INFINITY,
            max_expand: 1000,
            trust: false,
            // other options
            no_css: false,
            include_src: false,
            macros: None,
            block_delimiter: Delimiter::same("$$".into()),
            inline_delimiter: Delimiter::same("$".into()),
            pre_render: true,
        }
    }
}

impl KatexConfig {
    /// Generate extra options for the preprocessor.
    pub fn build_extra_opts(&self) -> ExtraOpts {
        ExtraOpts {
            include_src: self.include_src,
            block_delimiter: self.block_delimiter.clone(),
            inline_delimiter: self.inline_delimiter.clone(),
        }
    }
}

/// Extract configuration for katex preprocessor from `book_cfg`.
pub fn get_config(
    book_cfg: &mdbook_preprocessor::config::Config,
) -> Result<KatexConfig, toml::de::Error> {
    let cfg = match book_cfg
        .get::<toml::Value>("preprocessor.katex")
        .unwrap_or_default()
    {
        Some(raw) => raw.clone().try_into(),
        None => Ok(KatexConfig::default()),
    };
    cfg.or_else(|_| Ok(KatexConfig::default()))
}
