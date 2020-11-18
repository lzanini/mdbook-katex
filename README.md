All KaTex equations are converted to html when the book is compiled, which allows for faster page rendering.

# Basic Usage

First, install the crate

```
cargo install mdbook-katex
```

Then, add the KaTex preprocessor to your book.toml file

```toml
[preprocessor.katex]
command = "mdbook-katex"
```

Once this is done, you can use KaTex expressions within your `.md` files, using `$` and `$$` delimiters.

```
# Chapter 1

Here is an inline example, $ \pi(\theta) $, and here is an equation:

$$ \nabla f(x) \in \mathbb{R}^n $$
```

# Macros

Only KaTex macros with no argument are supported for now. They must be specified in a `.txt` file, according to the following pattern

```txt
\grad:{\nabla}
\Rn:{\mathbb{R}^n}
```

Then, change the preprocessor command to tell it where the macros are located

```toml
[preprocessor.katex]
command = "mdbook-katex --macros=./macros.txt"
```

You can then use these macros in any `.md` file.

```
# Chapter 1

Here is an inline example, $ \pi(\theta) $, and here is an equation:

$$ \grad f(x) \in \Rn $$```
