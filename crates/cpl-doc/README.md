# cpl-doc

`cpl doc` writes documentation from the declarations and `///` comments of
`.cp` files (`plan/45-dev-tools.md`): a static site, or JSON for another
generator.

```
/// The sum of `a[l..r]`.
///
/// ```
/// let p = Prefix.of([3, 1, 4])
/// print(p.sum(0, 2))
/// ```
@cost("O(1)")
fn sum(self, l: Int, r: Int) -> Int = self.pre[r] - self.pre[l]
```

```
$ cpl doc lib/ --title "My library"
documented 11 items in 1 file, 3 without a comment; open target/doc/index.html
```

Each item's page entry shows the signature as written, its `@cost` as a
badge, the comment rendered from Markdown, and a link to its source line
when `--source-url` is given. The pages are plain HTML and one stylesheet:
they load nothing from the network and open straight from disk.

## What is documented

Every `fn`, `struct` (with its fields and methods), `enum` (with its
variants), `trait`, `impl`, `domain`, `type` and `const` of a file, in
source order, whether or not it has a comment. A top-level `let` is listed
only when it has one, since most are a program's input and not a library's
interface. A name starting with `_` is a helper and is left out.

A `///` comment documents the declaration directly below it, above any
decorators. A `///` block at the top of a file followed by a blank line
documents the file. Comments are Markdown: paragraphs, `#` headings, `-`
lists, fenced code, and inline `code`, `*emphasis*` and `[links](url)`.

## Examples are checked

A fenced block with no language (or `cp`) is an example. `cpl doc --test`
parses every one and fails on any that is not a program, with the error
shown against the example's own text:

```
$ cpl doc --test lib/
FAIL lib/prefix.cp:46: the example for `larger` does not parse
error[E0101]: this `(` is never closed
2 examples checked, 1 failed
```

Running the examples and comparing their output waits on the interpreter;
until then a passing check means the example is syntactically valid, not
that it prints what the comment says. Blocks marked with another language
(` ```text `, ` ```cpp `) are shown and not checked.

## Other output

`cpl doc --format json` prints one document on stdout, with the same
fields the pages are built from (`kind`, `name`, `signature`, `doc`,
`summary`, `cost`, `line`, `examples`, `members`), so the documentation
site can render stdlib reference pages with its own templates and never
drift from the code.

## Layout

- `src/model.rs`: `extract(src, path) -> Module`, the items and how
  comments attach to them.
- `src/markdown.rs`, `src/highlight.rs`: comments and code to HTML.
- `src/html.rs`, `src/style.css`: the site.
- `src/json.rs`: the JSON form.
- `src/examples.rs`: what `--test` runs.
- `src/cli.rs`: the `cpl doc` command.

```
$ cargo test -p cpl-doc
```
