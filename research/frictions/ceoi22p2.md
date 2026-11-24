# ceoi22p2 frictions

## Dream

```cp
let e: Expr = parse(read<Str>(), grammar: "min(E,E) | max(E,E) | ?")
fn need(e: Expr) -> Need = match e: ...
```

## Expression parsing by hand

There is no std parser for a small expression grammar, so the port runs the shunting stack by
hand, telling `min` and `max` apart by their middle letter (`'i'` only in min, `'a'` only in max).
