//! Configurations for preprocessing KaTeX.

#[cfg(feature = "pre-render")]
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};

use crate::{preprocess::ExtraOpts, scan::Delimiter};

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
            pre_render: cfg!(feature = "pre-render"),
        }
    }
}

impl KatexConfig {
    /// Configured output type.
    /// Defaults to `Html`, can also be `Mathml` or `HtmlAndMathml`.
    #[cfg(feature = "pre-render")]
    pub fn output_type(&self) -> katex::OutputType {
        match self.output.as_str() {
            "html" => katex::OutputType::Html,
            "mathml" => katex::OutputType::Mathml,
            "htmlAndMathml" => katex::OutputType::HtmlAndMathml,
            other => {
                eprintln!(
"[preprocessor.katex]: `{other}` is not a valid choice for `output`! Please check your `book.toml`.
Defaulting to `html`. Other valid choices for output are `mathml` and `htmlAndMathml`."
                );
                katex::OutputType::Html
            }
        }
    }

    /// From `root`, load macros and generate configuration options
    /// `(inline_opts, display_opts, extra_opts)`.
    #[cfg(feature = "pre-render")]
    pub fn build_opts<P>(&self, root: P) -> (katex::Opts, katex::Opts)
    where
        P: AsRef<Path>,
    {
        // load macros as a HashMap
        let macros = load_macros(root, &self.macros);

        self.build_opts_from_macros(macros)
    }

    /// Given `macros`, generate `(inline_opts, display_opts)`.
    #[cfg(feature = "pre-render")]
    pub fn build_opts_from_macros(
        &self,
        macros: HashMap<String, String>,
    ) -> (katex::Opts, katex::Opts) {
        let mut configure_katex_opts = katex::Opts::builder();
        configure_katex_opts
            .output_type(self.output_type())
            .leqno(self.leqno)
            .fleqn(self.fleqn)
            .throw_on_error(self.throw_on_error)
            .error_color(self.error_color.clone())
            .macros(macros)
            .min_rule_thickness(self.min_rule_thickness)
            .max_size(self.max_size)
            .max_expand(self.max_expand)
            .trust(self.trust);
        // inline rendering options
        let inline_opts = configure_katex_opts
            .clone()
            .display_mode(false)
            .build()
            .unwrap();
        // display rendering options
        let display_opts = configure_katex_opts.display_mode(true).build().unwrap();
        (inline_opts, display_opts)
    }
    /// generate `extraOpts`
    pub fn build_extra_opts(&self) -> ExtraOpts {
        let extra_opts = ExtraOpts {
            include_src: self.include_src,
            block_delimiter: self.block_delimiter.clone(),
            inline_delimiter: self.inline_delimiter.clone(),
        };
        return extra_opts;
    }
}

/// Load macros from `root`/`macros_path` into a `HashMap`.
#[cfg(feature = "pre-render")]
fn load_macros<P>(root: P, macros_path: &Option<String>) -> HashMap<String, String>
where
    P: AsRef<Path>,
{
    // load macros as a HashMap
    let mut map = HashMap::new();
    if let Some(path) = get_macro_path(root, macros_path) {
        let macro_str = load_as_string(&path);
        for couple in macro_str.split('\n') {
            // only consider lines starting with a backslash
            if let Some('\\') = couple.chars().next() {
                let couple: Vec<&str> = couple.splitn(2, ':').collect();
                map.insert(String::from(couple[0]), String::from(couple[1]));
            }
        }
    }
    map
}

/// Absolute path of the macro file.
#[cfg(feature = "pre-render")]
pub fn get_macro_path<P>(root: P, macros_path: &Option<String>) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    macros_path
        .as_ref()
        .map(|path| root.as_ref().join(PathBuf::from(path)))
}

/// Extract configuration for katex preprocessor from `book_cfg`.
pub fn get_config(book_cfg: &mdbook::Config) -> Result<KatexConfig, toml::de::Error> {
    let cfg = match book_cfg.get("preprocessor.katex") {
        Some(raw) => raw.clone().try_into(),
        None => Ok(KatexConfig::default()),
    };
    cfg.or_else(|_| Ok(KatexConfig::default()))
}

/// Read file at `path`.
#[cfg(feature = "pre-render")]
pub fn load_as_string(path: &Path) -> String {
    let display = path.display();

    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {display}: {why}"),
        Ok(file) => file,
    };

    let mut string = String::new();
    if let Err(why) = file.read_to_string(&mut string) {
        panic!("couldn't read {display}: {why}")
    };
    string
}
