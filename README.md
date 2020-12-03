A preprocessor for [mdBook](https://github.com/rust-lang/mdBook), pre-rendering LaTex equations to HTML at build time. It allows for very fast page loading, compared to rendering equations in the browser.

This preprocessor uses the [Katex](https://github.com/xu-cheng/katex-rs) crate; see [this page](https://katex.org/docs/supported.html) for the list of supported Latex functions.

<p align="center">
  <img width="75%" height="75%" src="https://raw.githubusercontent.com/lzanini/mdbook-katex/master/katex_mathjax.gif">
</p>

## Getting Started

First, install the `mdbook-katek` crate

```
cargo install mdbook-katex
```

Then, add the following line to your `book.toml` file

```toml
[preprocessor.katex]
```

You can now use `$` and `$$` delimiters for inline / display equations within your `.md` files. If you need a regular dollar symbol, you can escape delimiters with a backlash `\$`.

```
# Chapter 1

Here is an inline example, $ \pi(\theta) $, 

an equation,

$$ \nabla f(x) \in \mathbb{R}^n, $$

and a regular \$ symbol.
```

Latex equations will be pre-rendered as HTML when running `mdbook build` or `mdbook serve` as usual.

## Macros

Latex macros are supported. They must be defined in a `.txt` file, according to the following pattern

```txt
\grad:{\nabla}
\R:{\mathbb{R}^{#1 \times #2}}
```

You must specify the path of this file as an option under the `preprocessot.katex` table of your `book.toml` file.

```toml
[preprocessor.katex]
macros = "path/to/macros.txt"
```

These macros can then be used in your `.md` files

```
# Chapter 1

$$ \grad f(x) \in \R{n}{p} $$
```
