# Friction report: ioi16p5

## Dream

```cp
let groups = Pos.collect().split_by_levels(Level, (g, d) => g.partition(a => a.bit(d)))
```

## Recursion becomes levels

The v0.2 port had two recursive phases with an `Int` list per group, a hand-built
`in_cur` mask and `(a / (1 << bit)) % 2` bit tests. The rewrite walks the levels
iteratively. Groups are `List<Pos>` split by `partition`, and both phases share one `shape`
string. The write phase splits on the known index bit, and the read phase splits on the
membership answer.

## Interactive package

There was no generator or judge before. `gen.py` makes the hidden permutation (n = 2^b,
w = r = n b, the hardest subtask at `--max`). `interactor.py` enforces the w/r limits and the
single compile, applies y[i] = x[p[i]] and checks the answer. The accepted C++ is grader-only,
so the interactor is the reference.

## Verification

30 interactive cases pass for the port and for the v0.2 port. Corpus tests pass 10/10,
including the n = 128 max case.
