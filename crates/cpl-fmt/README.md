# cpl-fmt

`cpl fmt` rewrites a `.cp` file in the one canonical style
(`plan/45-dev-tools.md`). There are no options: four spaces per level,
lines up to 100 columns, one space around binary operators, none inside
brackets.

```
$ printf 'let a:[I]1..=1e9=read( )\nfor i in I :\n  best max=a[i]+1 // grow\n' | cpl fmt --stdin
let a: [I]1..=1e9 = read()
for i in I:
    best max= a[i] + 1  // grow
```

`cpl fmt` with no path formats every `.cp` file under the current
directory; `cpl fmt --check` changes nothing, lists the files that would
change, and exits 1 if there are any.

## What it does

- Indentation becomes four spaces per block. A bracket that does not fit
  in 100 columns is opened up, one element per line with a trailing comma,
  outermost bracket first:

  ```
  fn solve(
      first_argument: Int,
      second_argument: List<Int>,
  ) -> Int:
  ```

  A bracket that fits is joined back onto one line. A line that is too long
  and has no bracket to open is left alone.
- Comments stay attached to the line or the element they were on. Trailing
  comments on consecutive lines are lined up, two spaces after the longest.
- Runs of blank lines become one; the file ends with one newline.

## What it guarantees

The formatter works on the lexer's tokens, not the syntax tree, so it
formats a file the parser rejects, and every comment survives. `format`
re-lexes its own output and refuses to return it unless the tokens are the
same as the input's, apart from a trailing comma added or dropped before a
closing bracket; `(x,)` keeps its comma. A file the lexer cannot read (an
unclosed bracket, a tab in indentation) is reported and not touched.

Formatting twice gives the same text as formatting once. The tests check
that, and that the parser's verdict does not change, on every `.cp` file
under `proto/`.

Two pairs read both ways without a parse, and there the source's own
spacing is kept: `[V]Int` (a type) against `[I] i => 0` (a table), and
`x? + y` (postfix `?`) against `x ? y`.

## Layout

- `src/lib.rs`: `format`, `is_formatted`, and the token check.
- `src/spacing.rs`: whether a space goes between two tokens.
- `src/layout.rs`: logical lines, comments, and bracket breaking.
- `src/cli.rs`: the `cpl fmt` command.

```
$ cargo test -p cpl-fmt
```
