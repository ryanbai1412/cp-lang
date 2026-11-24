# Friction Report: ccc20s5

## A suffix-sum DP needs a running variable
- **Where:** `var after = 0.0` beside `chance[i]`, updated at the end of each step.
- **What felt wrong:** `chance[i]` reads the sum of `chance` after `i`; the port keeps that sum
  by hand in a second variable that must be updated after each entry.
- **Goal it hurts:** common mistakes hard to write.
- **Ideal code:** `let chance = Pos.rev.scan_build((i, later) => ... later.sum ...)`.
- **General fix:** a reverse array builder that exposes the running suffix fold.
- **Severity:** low

## Dream

```cp
let n: 1..=1e6 = read()
domain Pos = 0..n
domain Kind = 1..=5e5
let fav: [Pos]Kind = read()
let last_of = fav.last_index_by_value()
let chance = Pos.build_rev((i, later) =>
    if fav[i] == fav.first!: 1.0 elif fav[i] == fav.last!: 0.0
    elif last_of[fav[i]] == i: (1.0 + later.sum) / F64(n - i.idx) else: later[last_of[fav[i]]])
print(if fav.first! == fav.last!: 1.0 else: chance.sum / F64(n), precision: 8)
```

## Measurements

- Tokens: port 205, original 562
- Max input (`gen.py --max`): port 0.02 s, 17616 KB, original 0.07 s, 27136 KB
- Emitted C++: 9881 bytes
