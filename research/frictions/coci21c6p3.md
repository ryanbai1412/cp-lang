# Friction report: coci21c6p3

Rethought. The v0.2 port built `out_adj`/`in_adj` and both degree arrays edge by edge, threaded `mut seen, mut que` through a `try_push` helper, and walked lists by index. It checked completeness with an `all_done` flag loop, encoded each step as an anonymous `(V, Int)`, and printed the reverse with a `while i >= 0` countdown. The v0.3 port reads the wires at once into `Graph.directed` and takes the incoming side from `transpose()`. `offer` keeps the peel stack, the steps are `(ball:, charge: 0..=1)`, completeness is `Ball.all(..)`, and the steps print through `order.rev`. The package gains a checker.py, since any valid charging order is accepted.

No language gap.

## Measurements

gen.py --max: port 0.12 s, a valid answer. The accepted C++ recurses and dies (signal 11) under an 8 MB stack; with an unlimited stack it runs in 0.10 s.
