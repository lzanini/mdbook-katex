A Rust pre-processor for [mdBook](https://github.com/rust-lang/mdBook), converting Latex equations to HTML at compile time. This preprocessor uses the [Katex](https://github.com/xu-cheng/katex-rs) crate; see [this page](https://katex.org/docs/supported.html) for the list of supported Latex functions.


<p align="center">
  <img width="70%" height="70%" src="https://raw.githubusercontent.com/lzanini/mdbook-katex/master/katex_mathjax.gif">
</p>

## Usage

Install the crate

```
cargo install mdbook-katex
```

Add the Katex preprocessor to your `book.toml` file

```toml
[preprocessor.katex]
```

You can then use KaTex expressions within your `.md` files, using `$` and `$$` delimiters. 

Use `\$` for a regular dollar symbol.

```
# Chapter 1

Here is an inline example, $ \pi(\theta) $, 

an equation,

$$ \nabla f(x) \in \mathbb{R}^n, $$

and a regular \$ symbol.
```

## Macros

Macros with no arguments are supported. They must be defined in a `.txt` file, according to the following pattern

```txt
\grad:{\nabla}
\Rn:{\mathbb{R}^n}
```

Then, specify the macros location in your `book.toml`

```toml
[preprocessor.katex]
macros = "path/to/macros.txt"
```

You can now use these macros in any `.md` file

```
# Chapter 1

$$ \grad f(x) \in \Rn $$
```
