//! Configurations for preprocessing KaTeX.

use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::cfg::KatexConfig;

impl KatexConfig {
    /// Configured output type.
    /// Defaults to `Html`, can also be `Mathml` or `HtmlAndMathml`.
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
    /// `(inline_opts, display_opts)`.
    pub fn build_opts<P>(&self, root: P) -> (katex::Opts, katex::Opts)
    where
        P: AsRef<Path>,
    {
        // load macros as a HashMap
        let macros = load_macros(root, &self.macros);

        self.build_opts_from_macros(macros)
    }

    /// Given `macros`, generate `(inline_opts, display_opts)`.
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
}

/// Load macros from `root`/`macros_path` into a `HashMap`.
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
pub fn get_macro_path<P>(root: P, macros_path: &Option<String>) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    macros_path
        .as_ref()
        .map(|path| root.as_ref().join(PathBuf::from(path)))
}

/// Read file at `path`.
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
