# Friction Report: ccc19s4

The v0.2 port checked `assert k <= n` after reading, walked the blocks with a `while lo < n`
loop and `lo = hi`, and its `@alt` brute seeded the day count with the sentinel `n + 1` and
compared candidates field by field. Now `k: 1..=n` carries the bound, the blocks are
`(0..n).step(k)` with a named `block` range of cuts, scores have one `Score` type, and the
brute is one `min` over `(days, loss)` records per cut with no sentinel. The package gains
tests.toml.

## Each cut is converted from Int at every index
- **Where:** `best[Cut(i)]`, `plain[Cut(j)]`, `score[Spot(i)]` inside both block loops.
- **Ideal:** blocks as ranges of `Cut` (`Cut.chunks(k)`), with `Spot` reached from a cut by
  `c.spot` or a shared offset, so the loops run over domain values.

## "Largest total" is spelled as the smallest loss
- **Where:** the brute's `loss` field, so `min` can order by fewest days then largest total.
- **Ideal:** `min(by: p => (p.days, desc(p.total)))`.

## Dream
```cp
fn most(score: [Spot]Score) -> Int:
    var best: [Cut]Int = 0
    var (plain, tailed): ([Cut]Int, [Cut]Int) = (0, 0)
    for block in Cut.after_first.chunks(k):
        for (i, head) in block.zip(score[block].prefix_max):
            let j = max(i - k, Cut.first!)
            best[i] = if block.is_first: head else: max(tailed[j], plain[j] + head)
        plain[block] = best[block].suffix_max
        tailed[block] = (best[block] + score[block].suffix_max.shift(1, 0)).suffix_max
    best[Cut.last!]
```

- Tokens: port 421, original 1499
- Max input (`gen.py --max`): port 0.03 s, 32720 KB, original 0.13 s, 23412 KB
