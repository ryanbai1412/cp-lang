# Friction report: cpc21c1p5

Already close to the bar. The v0.3 port names the coordinate limit once (`SIDE`; it was written four times), drops the unused loop variable, and removes the "as it builds today" header note.

## Gap

`ask` only works as a `let` initializer. `ask<0..=500>(...) - count(...)` in expression position fails with E0501 (`ask` is not in scope during lowering), so `left` keeps a `let inside` of its own. Repro sent to Core.

## Dream

```
fn left(x1: Int, x2: Int, y1: Int, y2: Int) -> Int =
    ask<0..=500>("?", x1, x2, y1, y2) -
        count(x1 <= b.x1 and b.x2 <= x2 and y1 <= b.y1 and b.y2 <= y2 for b in found)
```
