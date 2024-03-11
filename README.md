# mdBook-KaTeX

[![Crates.io version](https://img.shields.io/crates/v/mdbook-katex)](https://crates.io/crates/mdbook-katex)
![Crates.io downloads](https://img.shields.io/crates/d/mdbook-katex)

mdBook-KaTeX is a preprocessor for [mdBook](https://github.com/rust-lang/mdBook), pre-rendering LaTeX math expressions to HTML at build time.

- Very fast page loading. Much faster than rendering equations in the browser.
- Pre-rendered KaTeX formulas, no need for client-side JavaScript.
- Customization such as macros and delimiters.

Pre-rendering uses [the katex crate](https://github.com/xu-cheng/katex-rs).
[List of LaTeX functions supported by KaTeX](https://katex.org/docs/supported.html).

<p align="center">
  <img width="75%" height="75%" src="https://user-images.githubusercontent.com/71221149/107123378-84acbf80-689d-11eb-811d-26f20e32556c.gif">
</p>

## Getting Started

First, install mdBook-KaTeX

### **Non-Windows** users

```shell
cargo install mdbook-katex
```

### Windows users

> The recommended way is to download the latest `x86_64-pc-windows-gnu.zip` from [Releases](https://github.com/lzanini/mdbook-katex/releases) for the full functionality, otherwise, things such matrices will not work fine. See [#67](https://github.com/lzanini/mdbook-katex/issues/67) for the reasons.
>
> Another way is [Escaping mode](#escaping-mode).

Then, add the following line to your `book.toml` file

```toml
[preprocessor.katex]
after = ["links"]
```

You can now use `$` and `$$` delimiters for inline and display math expressions within your `.md` files. If you need a regular dollar symbol, you need to escape delimiters with a backslash `\$`.

```markdown
# Chapter 1

Here is an inline example, $ \pi(\theta) $,

an equation,

$$ \nabla f(x) \in \mathbb{R}^n, $$

and a regular \$ symbol.
```

Math expressions will be rendered as HTML when running `mdbook build` or `mdbook serve` as usual.

## KaTeX options

Most [KaTeX options](https://katex.org/docs/options.html) are supported via the `katex` crate.
Specify these options under `[preprocessor.katex]` in your `book.toml`:

| Argument                                                                                            | Type                                       |
| :-------------------------------------------------------------------------------------------------- | :----------------------------------------- |
| [`output`](https://katex.org/docs/options.html#:~:text=default-,output,-string)                     | `"html"`, `"mathml"`, or `"htmlAndMathml"` |
| [`leqno`](https://katex.org/docs/options.html#:~:text=default-,leqno,-boolean)                      | `boolean`                                  |
| [`fleqn`](https://katex.org/docs/options.html#:~:text=LaTeX-,fleqn,-boolean)                        | `boolean`                                  |
| [`throw-on-error`](https://katex.org/docs/options.html#:~:text=package-,throwonerror,-boolean)      | `boolean`                                  |
| [`error-color`](https://katex.org/docs/options.html#:~:text=errorColor-,errorcolor,-string)         | `string`                                   |
| [`min-rule-thickness`](https://katex.org/docs/options.html#:~:text=state-,minrulethickness,-number) | `number`                                   |
| [`max-size`](https://katex.org/docs/options.html#:~:text=true-,maxsize,-number)                     | `number`                                   |
| [`max-expand`](https://katex.org/docs/options.html#:~:text=maxexpand)                               | `number`                                   |
| [`trust`](https://katex.org/docs/options.html#:~:text=LaTeX-,trust,-boolean)                        | `boolean`                                  |

There are also extra options to configure the behaviour of the preprocessor:

| Option             | Description                                                                                               |
| :----------------- | :-------------------------------------------------------------------------------------------------------- |
| `no-css`           | Do not inject KaTeX stylesheet link (See [Self-host KaTeX CSS and fonts](#self-host-katex-css-and-fonts)) |
| `macros`           | Path to macros file (see [Custom macros](#custom-macros))                                                 |
| `include-src`      | Include math expressions source code (See [Including math Source](#including-math-source))                |
| `block-delimiter`  | See [Custom delimiter](#custom-delimiter)                                                                 |
| `inline-delimiter` | See [Custom delimiter](#custom-delimiter)                                                                 |
| `pre-render`       | See [Escaping mode](#escaping-mode)                                                                       |

For example, the default configuration:

```toml
[preprocessor.katex]
after = ["links"]
# KaTeX options.
output = "html"
leqno = false
fleqn = false
throw-on-error = true
error-color = "#cc0000"
min-rule-thickness = -1.0
max-size = "Infinity"
max-expand = 1000
trust = false
# Extra options.
no-css = false
include-src = false
block-delimiter = { left = "$$", right = "$$" }
inline-delimiter = { left = "$", right = "$" }
pre-render = false
```

## Self-host KaTeX CSS and fonts

KaTeX requires a stylesheet and fonts to render correctly.

By default, mdBook-KaTeX injects a KaTeX stylesheet link pointing to a CDN.

If you want to self-host the CSS and fonts instead, you should specify in `book.toml`:

```toml
[preprocessor.katex]
no-css = true
```

and manually add the CSS and fonts to your mdBook project before building it.

See [mdBook-KaTeX Static CSS Example](https://github.com/SichangHe/mdbook_katex_static_css) for an automated example.

## Custom macros

Custom LaTeX macros must be defined in a `.txt` file, according to the following pattern

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

```markdown
# Chapter 1

$$ \grad f(x) \in \R{n}{p} $$
```

## Including math source

This option is added so users can have a convenient way to copy the source code of math expressions when they view the book.

When `include-src` is set to `true`, each math block is wrapped within a `<data>` tag with `class="katex-src"` with the included math source code being its `value` attribute.

For example, before being fed into mdBook,

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

<data class="katex-src" value="&#10;f(x)=x^2\\&#10;x\in\R&#10;"><span class="katex-display"><span class="katex">…</span></span></data>
```

The math source code is included in a minimal fashion, and it is up to the users to write custom CSS and JavaScript to make use of it.
For more information about adding custom CSS and JavaScript in mdBook, see [additional-css and additional-js](https://rust-lang.github.io/mdBook/format/configuration/renderers.html#html-renderer-options).

If you need more information about this feature, please check the issues or file a new issue.

## Custom delimiter

To change the delimiters for math expressions, set the `block-delimiter` and `inline-delimiter` under `[preprocessor.katex]`.
For example, to use `\(`and `\)` for inline math and `\[` and `\]` for math block, set

```toml
[preprocessor.katex]
block-delimiter = { left = "\\[", right = "\\]" }
inline-delimiter = { left = "\\(", right = "\\)" }
```

Note that the double backslash above are just used to escape `\` in the TOML format.

## Caveats

`$\backslash$` does not work, but you can use `$\setminus$` instead.

Only the x86_64 Linux, Windows GNU, and macOS builds have full functionality (matrix, ...) , all other builds have compromised capabilities. See [#39](https://github.com/lzanini/mdbook-katex/issues/39) for the reasons.

## Escaping mode

"Escaping mode" is a beta feature that escapes the string needed for a formula in advance so that it remains the original formula after the md processor. This mode requires client-side js. May be useful for those who have problems with quickjs.

Disable pre-render to use "Escaping mode". Don't forget to include `katex.js` (you can include it in [index.hbs](https://rust-lang.github.io/mdBook/format/theme/index-hbs.html)).

```toml
[preprocessor.katex]
pre-render = false

[output.html]
theme = "theme" # use theme/index.hbs
```

A example index.hbs:

```html
<link rel="stylesheet" href="https://unpkg.com/katex@latest/dist/katex.min.css">
<script defer src="https://unpkg.com/katex@latest/dist/katex.min.js"></script>
<script defer src="https://unpkg.com/katex@latest/dist/contrib/auto-render.min.js"></script>

<script>
document.addEventListener("DOMContentLoaded", function () {
  renderMathInElement(document.body, {
    delimiters: [
      { left: '$$', right: '$$', display: true },
      { left: '$', right: '$', display: false },
    ],
  });
});
</script>
```
