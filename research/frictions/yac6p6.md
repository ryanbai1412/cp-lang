# yac6p6 frictions

## Dream

Each cell holds a `Costs = [Zeros]Int` table and one `enter(next: Costs, cost) -> Costs` plus
`either(x, y)` express the step for both directions:

```
back[r, c] = enter(either(back[r.next!, c], back[r, c.next!]), a[r, c])
```

## Gap

`[Zeros]Int` with a constant-size domain lowers to `std::vector<ll>`, so a table per cell
heap-allocates on every step: that spelling ran 1.16 s at --max (limit 1000 ms). The port keeps
flat `[Row, Col, Zeros]Int` tables with an `onward`/`before` helper per direction. Also
`g[r, c, k]` and `g[r, c][k]` on a `[R, C][Zeros]Int` are each rejected, each pointing at the
other (E0305/E0503). Both sent to Core.
