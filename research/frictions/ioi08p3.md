# Friction Report: ioi08p3

## Dream

```cp
domain Rank = Kind.sort(by: g => longest[g])   // a domain ordered by a key, with rank_of free
let free = top[r.next..].first_cut(t => t >= 2 * big)
total += lower * (Z(small[r]) + choices.fold(r.next..free))
```

## Gaps

- **A ranking is a domain plus two hand-built maps.** Ranking the kinds by
  their longest fish needs `order: List<Kind>`, `rank_of: [Kind]Rank` (filled
  with a placeholder first) and `top: [Rank]Int`; a "domain sorted by key"
  would give the ranks and both maps. Severity: medium.
- **Binary search over a domain suffix goes through `Int`.** The last kind
  freed by r's big fish is `(r.ord + 1..kinds).first_true(...)` and comes back
  as `Rank.cuts(free)`. Severity: low.
