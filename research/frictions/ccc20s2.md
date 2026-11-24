# Friction Report: ccc20s2

## Grid cells walked as nested loops
- **Where:** `for r in R: for c in C: leads[Num(r.label * c.label)].push(room[r, c])`.
- **What felt wrong:** the edge list is "each cell links its product to its value"; two nested
  loops spell a walk over one product domain.
- **Goal it hurts:** brevity.
- **Ideal code:** `let leads = group((Num(r.label * c.label), room[r, c]) for (r, c) in (R, C), over: Num)`.
- **General fix:** a `group(pairs, over: D) -> [D]List<T>` builder for adjacency.
- **Severity:** low

## Dream

```cp
with io(answers: ("yes", "no"))
let (rows, cols): (1..=1000, 1..=1000) = read()
domain R = 1..=rows
domain C = 1..=cols
domain Num = 1..=1e6
let room: [R, C]Num = read()
let leads = group((Num(r.label * c.label), room[r, c]) for (r, c) in (R, C), over: Num)
let exit = Num(rows * cols)
print(yes(bfs(leads, from: [room[R.first!, C.first!]], until: exit).reached(exit)))
```

## Measurements

- Tokens: port 177, original 514
- Max input (`gen.py --max`): port 0.08 s, 48592 KB, original 0.10 s, 40320 KB
- Emitted C++: 14029 bytes
