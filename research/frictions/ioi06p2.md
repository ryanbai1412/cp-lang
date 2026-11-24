# Friction Report: ioi06p2

## Dream

```cp
domain Corner = (1..=h - ch + 1, 1..=w - cw + 1)
let chamber: [Corner]Int = [Corner] (r, c) => pre.fold(r..r + ch, c..c + cw)
let best = chamber.window_fold((bh - ch - 1, bw - cw - 1), min)   // a 2D sliding minimum
```

## Gaps

- **Corners are plain offsets.** The chamber and base corners range over
  sub-grids of `Row x Col` whose sizes depend on the input, so they are 0-based
  `Int` offsets into nested lists, and `block` rebuilds `Row`/`Col` values from
  them. A domain for "the cells where a k x l block fits" would keep them typed.
  Severity: low.
- **The 2D sliding minimum is two 1D passes.** `window_fold` slides along a
  sequence, so the port slides along rows and then along the columns of the
  result, transposing through list comprehensions. Severity: low.
