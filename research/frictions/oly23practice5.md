# Friction Report: oly23practice5

## Dream

```cp
let a: [0..n]1..=n = read()
let pen = Counter(a.alternating_runs().map(r => (r.pair, r.len / 2)))
let by_odd = (0..=n).sorted(by: y => desc(odd[y]))
print(min(
    pen.items.filter(((x, y), _) => x != y).map(((x, y), e) => cost(x, y) + e).min() ?? n,
    (0..=n).map(x => cost(x, by_odd.first(y => y != x and (x, y) not in pen)!)).min()!,
))
```

## Gaps

- **Alternating runs are scanned by hand.** The run of `a[j] == a[j - 2]`
  is a `while` over positions; a `runs(by:)` over pairs of positions would
  give each run and its length. Severity: low.
- **The first penalty-free `y` is a loop with `break`.** Not a gap:
  `[3, 4, 5].first(y => y > 3)!` prints 4 on a fresh `cpl run`, so the
  per-`x` choice is `by_odd.first(y => y != x and (x, y) not in pen)` today;
  the port owes the rewrite.
