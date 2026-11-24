# btoi18p3 frictions

## Dream

```cp
let (n, m, k, q): (1..=1e6, 1..=1e6, 1..=1e6, 1..=1e5) = read()
domain X = 0..n
domain Y = 0..m
domain Z = 0..k
type Cell = (X, Y, Z)
@memo fn height(c: Cell) -> Int = ask("? {c.0.label + 1} {c.1.label + 1} {c.2.label + 1}")
fn on_sheet(r: Rect<X, Y>, seen: ?(Cell, Int)) -> ?Cell
```

## Cells off the box

A neighbour of an edge cell reads as height 0, so cells are plain `(Int, Int, Int)` and
`height` range-checks them; a product domain with a `beside` that only yields cells inside
would make the guard and the `0` go away. Questions are memoised with an explicit `Map`
because there is no `@memo` on an `ask`. The sheet search passes its rectangle as four ints.
