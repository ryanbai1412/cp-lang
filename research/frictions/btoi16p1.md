# Friction report: btoi16p1

Rethought. The v0.2 port read n untyped and built a `[P]List<(to: P)>` by hand. It priced each root with a `pay = -1` sentinel and a `break`. The v0.3 port types n from the statement and reads the accepted bosses as `[P]List<P>`. It builds the boss -> report graph with `Graph.of`. `payroll(root) -> ?Int` is None when the root misses someone, and the answer is the `min` over the roots that reach everyone.

## Gap

A draft that handed `bfs` the closure `b => reports[b]` over a hand-built `[P]List<P>` took 0.56 s at --max, against 0.11 s for `Graph.of`. Nothing in the source shows that cost.

## Measurements

gen.py --max: port 0.11 s, accepted C++ 0.12 s, same output.
