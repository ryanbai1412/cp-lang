# Friction report: sac22cc2p3

There is no dream.cp; the ideal is below. The port builds `parent` and `fork_at` arrays from the fork table, makes a `Rooted.from_parents`, and runs one `t.up(max, lift:, finish:)`. `lift` adds the gain when the child is the fork's take child.

## Dream

```cp
let forks: [Fork](at: Node, take: Node, skip: Node, gain: -1e9..=1e9) = read()
let by_node = forks.index(by: .at)          // [Node]?Fork
@memo(array)
fn best(v: Node) -> Int =
    if let f = by_node[v]: max(forks[f].gain + best(forks[f].take), best(forks[f].skip)) else: 0
print(r + best(Node.first!))
```

## The fork table is rebuilt into a parent array to get a tree DP
- **Where:** lines 20-30: two `var` arrays filled in a loop, `Rooted.from_parents`, and a `lift` that asks `forks[f].take == c` to learn which edge it is on.
- **What felt wrong:** the input already says, per node, its two children and the gain on one edge. The port throws that shape away to fit `Rooted`, then recovers it in `lift`.
- **Goal it hurts:** express.
- **Ideal code:** the dream's memoized recursion over the fork table, or `forks.index(by: .at)` for the `[Node]?Fork` lookup.
- **General fix:** a std `index(by:)` from a table to an optional array over the key's domain, the inverse lookup that the port's `fork_at` loop builds by hand. The `@memo(array)` recursion is up to 2^20 deep on a path-shaped tree, which the emitted stack allows. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 230, accepted C++ 345 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.19 s / 35 MB, accepted C++ 0.05 s / 33 MB, with the same output.
- **Emitted C++:** 22391 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
