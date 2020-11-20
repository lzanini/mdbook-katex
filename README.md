A Rust preprocessor for [mdBook](https://github.com/rust-lang/mdBook), converting Latex equations to HTML at compile time. It allows for very fast page loading, compared to rendering the equations in the browser.

This preprocessor uses the [Katex](https://github.com/xu-cheng/katex-rs) crate; see [this page](https://katex.org/docs/supported.html) for the list of supported Latex functions.


<p align="center">
  <img width="70%" height="70%" src="https://raw.githubusercontent.com/lzanini/mdbook-katex/master/katex_mathjax.gif">
</p>

## Usage

Install the crate

```
cargo install mdbook-katex
```

Add the preprocessor to your `book.toml` file

```toml
[preprocessor.katex]
```

Use `$` and `$$` delimiters for inline / display equations within your `.md` files. Use `\$` for a regular dollar symbol.

```
# Chapter 1

Here is an inline example, $ \pi(\theta) $, 

an equation,

$$ \nabla f(x) \in \mathbb{R}^n, $$

and a regular \$ symbol.
```

## Macros

Macros must be defined in a `.txt` file, according to the following pattern

```txt
\grad:{\nabla}
\R:{\mathbb{R}^{#1 \times #2}}
```

Then, specify the macros location in your `book.toml`

```toml
[preprocessor.katex]
macros = "path/to/macros.txt"
```

You can now use these macros in your `.md` files

```
# Chapter 1

$$ \grad f(x) \in \R{n}{p} $$
```
