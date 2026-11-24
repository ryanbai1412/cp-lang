# btoi18p1 frictions

## Dream

```cp
let (rings, peel) = functional_graph(loves)
```

## Functional graph by hand

Not a gap for the cycles (corrected in review): std `Functional(nx).cycles()` (std/graph.cp)
lists every cycle in walk order. Run (b244fe76c), `domain P = 0..5` /
`let loves: [P]P = [1, 2, 0, 0, 3]` / `print(Functional(loves).cycles())` under `cpl run`
prints `0 1 2`. The port's stack walk of each ring is bookkeeping the language already
removes; only the leaf-first peel that carries the matching stays hand-written, and no std gap
is claimed for it.
