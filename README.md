A preprocessor for [mdBook](https://github.com/rust-lang/mdBook), pre-rendering LaTex equations to HTML at build time. It allows for very fast page loading, compared to rendering equations in the browser.

This preprocessor uses the [katex](https://github.com/xu-cheng/katex-rs) crate; see [this page](https://katex.org/docs/supported.html) for the list of supported LaTex functions.

<p align="center">
  <img width="75%" height="75%" src="https://user-images.githubusercontent.com/71221149/107123378-84acbf80-689d-11eb-811d-26f20e32556c.gif">
</p>

## Getting Started

First, install the `mdbook-katex` crate

```
cargo install mdbook-katex
```

Then, add the following line to your `book.toml` file

```toml
[preprocessor.katex]
```

You can now use `$` and `$$` delimiters for inline and display equations within your `.md` files. If you need a regular dollar symbol, you can escape delimiters with a backslash `\$`.

```
# Chapter 1

Here is an inline example, $ \pi(\theta) $, 

an equation,

$$ \nabla f(x) \in \mathbb{R}^n, $$

and a regular \$ symbol.
```

LaTex equations will be rendered as HTML when running `mdbook build` or `mdbook serve` as usual.

## Custom macros

Custom LaTex macros must be defined in a `.txt` file, according to the following pattern

```txt
\grad:{\nabla}
\R:{\mathbb{R}^{#1 \times #2}}
```

You need to specify the path of this file in your `book.toml` as follows

```toml
[preprocessor.katex]
macros = "path/to/macros.txt"
```

These macros can then be used in your `.md` files

```
# Chapter 1

$$ \grad f(x) \in \R{n}{p} $$
```
