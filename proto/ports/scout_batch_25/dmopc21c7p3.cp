// DMOPC '21 Contest 7 P3 (DMOJ: dmopc21c7p3)
// SOURCE-FIRST: the DMOJ API and statement page were both behind a Cloudflare
// challenge and this contest has no other published spec, so the task below
// is reconstructed from corpus/submissions/4453037.cpp, not from the statement.
//
// N lamps, each on or off, and M directed wires u -> v. Every second all lamps
// update at once: a lamp becomes the XOR of itself and the lamps wired into
// it. If a cycle can be reached from a lit lamp the process never settles:
// print -1. Otherwise print the number of lamps lit after one step.
let (n, m): (1..=1e6, 0..=1e6) = read()
domain V = 0..n
let lit: [V]Bool = [V] _ => read<Int>() == 1
let g = Graph.directed(V, read<[0..m](V, V)>(base: 1))

// A cycle reachable from a lit lamp: DFS colours over the lit-reachable part.
if has_cycle(g, from: [v for v in V if lit[v]]):
    print(-1)
    return

print(count(v for v in V if lit[v] ^ xor(lit[u] for u in g.into(v))))
