`mdbook-katex` is a preprocessor for [mdBook](https://github.com/rust-lang/mdBook), pre-rendering LaTex equations to HTML at build time. It allows for very fast page loading, compared to rendering equations in the browser.

This preprocessor uses the [katex](https://github.com/xu-cheng/katex-rs) crate; see [this page](https://katex.org/docs/supported.html) for the list of supported LaTex functions.

<p align="center">
  <img width="75%" height="75%" src="https://user-images.githubusercontent.com/71221149/107123378-84acbf80-689d-11eb-811d-26f20e32556c.gif">
</p>

## Getting Started

First, install `mdbook-katex`

```shell
cargo install mdbook-katex
```

Then, add the following lines to your `book.toml` file

```toml
[output.katex]

[preprocessor.katex]
renderers = ["html"]
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

## Katex options

The preprocessor supports passing options to the katex-rs crate in order
to configure its behaviour. These options are specified under the
`[preprocessor.katex]` directive.

The currently spported arguments are:
| Argument | Type |
| :- | :- |
| [`leqno`](https://katex.org/docs/options.html#:~:text=default-,leqno,-boolean) | `boolean` |
| [`fleqn`](https://katex.org/docs/options.html#:~:text=LaTeX-,fleqn,-boolean) | `boolean` |
| [`throw-on-error`](https://katex.org/docs/options.html#:~:text=package-,throwonerror,-boolean) | `boolean` |
| [`error-color`](https://katex.org/docs/options.html#:~:text=errorColor-,errorcolor,-string) | `string` |
| [`min-rule-thickness`](https://katex.org/docs/options.html#:~:text=state-,minrulethickness,-number) | `number` |
| [`max-size`](https://katex.org/docs/options.html#:~:text=true-,maxsize,-number) | `number` |
| [`max-expand`](https://katex.org/docs/options.html#:~:text=maxexpand) | `number` |
| [`trust`](https://katex.org/docs/options.html#:~:text=LaTeX-,trust,-boolean) | `boolean` |

There are also options to configure the behaviour of the preprocessor:
| Option | Default | Description |
| :- | :- | :- |
| `static-css` | `false` | Generates fully static html pages with katex styling |
| `macros` | `None` | Path to macros file (see [Custom macros](#custom-macros)) |
| `include-src` | `false` | Append the source code for the rendered math expressions after them |

For example:

```toml
[preprocessor.katex]
renderers = ["html"]
static-css = false
include-src = false
```

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

## Including math source

This option is added so users can have a convenient way to copy the source code of math expressions when they view the book.

When `include-src` is set to `true`, each math block is wrapped within a `<data>` tag with `class="katex-src"` with the included math source code being its `value` attribute.

For example, before being fed into `mdbook`,

```markdown
Define $f(x)$:

$$
f(x)=x^2\\
x\in\R
$$
```

is preprocessed into (the content of the `katex` `span`s are omitted and represented as `…`)

```markdown
Define <data class="katex-src" value="f(x)"><span class="katex">…</span></data>:

<data class="katex-src" value="
f(x)=x^2\\
x\in\R
"><span class="katex-display"><span class="katex">…</span></span></data>
```

The math source code is included in a minimal fashion, and it is up to the users to write custom CSS and JavaScript to make use of it.
For more information about adding custom CSS and JavaScript in `mdbook`, see [additional-css and additional-js](https://rust-lang.github.io/mdBook/format/configuration/renderers.html#html-renderer-options).

If you need more information about this feature, please check the issues or file a new issue.

## Caveats

The build artifact of the book will be in a folder named `html` inside the directory you specify instead of being directly there.
Consider this when you use `mdbook_katex` in your CIs.

`$\backslash$` does not work, but you can use `$\setminus$` instead.
