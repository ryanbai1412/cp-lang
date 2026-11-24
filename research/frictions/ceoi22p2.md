# ceoi22p2 frictions

## Dream

```cp
let e: Expr = parse(read<Str>(), grammar: "min(E,E) | max(E,E) | ?")
fn need(e: Expr) -> Need = match e: ...
```

## Expression parsing by hand

There is no std parser for a small expression grammar, so the port runs the shunting stack by
hand, telling `min` and `max` apart by their middle letter (`'i'` only in min, `'a'` only in max).

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
use std.strings.{parse}
let e = parse(read<Str>(), grammar: "min(E,E) | max(E,E) | ?")
print(e)
```

```text
error[E0310]: `parse` has no parameter named `grammar`
  --> probe.cp:2:28
2 | let e = parse(read<Str>(), grammar: "min(E,E) | max(E,E) | ?")
  |                            ^^^^^^^
  = help: its parameters are `self`
```

**Status:** open: std's `parse` reads a value with `read<T>`; there is no grammar parser.
