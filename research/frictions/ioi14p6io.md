# Friction Report: ioi14p6io

The port was already rethought around `monotone_minima` and std `WaveletMatrix` top-k sums.
Its two sides were comprehensions that walked every city and kept those before or after
`home` (`for c in City if c < home`). They are now the sub-ranges `fun[..home].rev` and
`fun[home..].drop(1)`, collected once. `fun[home.next!..]` would trap when the start is the
last city, so the right side drops `home` instead.

## A row-maximum search is spelled through minima
- **Where:** `seen` returns `Some(-(...))` and the result is negated back, `-b.val`.
- **Ideal:** `monotone_maxima` beside `monotone_minima` in std.dp.

## The sides are copied to fit WaveletMatrix and Prefix
- **Where:** `.collect()` on both slices; `away` takes `List<Int>` because `WaveletMatrix`
  needs `[D]Int` and `Prefix<Int>` a `List<Int>`.
- **Ideal:** both built from any `Indexed<Int>` view, so the sides stay views.

## Dream
```cp
fn away(side: Indexed<Int>, step: Int) -> List<Int>:
    let order = WaveletMatrix(side)
    let passed = Prefix(side)
    fn seen(t: Int, i: Int) -> ?Int:
        let visits = min(t - (i + 1) * step, i + 1)
        if visits < 0: None else: Some(order.top_sum(0..i + 1, visits))
    [best.val ?? 0 for best in monotone_maxima(days + 1, side.len, seen)]

print(max(best(fun[home..].drop(1), fun[..home].rev), best(fun[..home].rev, fun[home..].drop(1))))
```

- Tokens: port 364, original 1225
- Max input (`gen.py --max`): port 0.67 s, 33848 KB, original 2.08 s, 79520 KB
