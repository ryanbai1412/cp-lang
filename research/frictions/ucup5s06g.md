# Friction report: ucup5s06g

`dream.cp` is verified. The port is the dream with the statement's bounds (n <= 1e5, m <= 2e5 where the dream has 5e5) and `V.all(...)` for the dream's `all(... for v in V)`.

## Doubling the edges both ways is index arithmetic
- **Where:** port lines 23-26 (the dream has the same): `[Arc] k => let e = edges[E(Int(k) / 2)]; if Int(k) % 2 == 0: ... else: ...` over `domain Arc = 0..2 * m`.
- **What felt wrong:** "each edge forward at cost 0 and backward at cost 1" is encoded in the parity of an arc index, with `Int` conversions both ways.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** `Graph.directed(edges).with_reverse(forward: 0, backward: 1)`, or `edges.flat_map(e => [(e.u, e.v, 0), (e.v, e.u, 1)])` into a weighted graph.
- **General fix:** a std weighted builder that adds each edge's reverse with its own weight. `bfs01` over a "flip costs 1" graph is a common shape. Owner: stdlib.
- **Severity:** low.

## A list of peeled vertices becomes a `Bool` array by a loop
- **Where:** lines 20-21: `var peeled: [V]Bool = false; for v in peel(...): peeled[v] = true`.
- **What felt wrong:** a set membership table is built by hand from a list.
- **Goal it hurts:** express.
- **Ideal code:** `let peeled = Bitset<V>.of(peel(...))`.
- **General fix:** none in std if `Bitset.of` exists; the rewrite is owed by both dream and port and waits on the next dream verify. Owner: idioms.
- **Severity:** low.

## Measurements

- **Tokens:** port 234, accepted C++ 647 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.02 s / 19 MB, accepted C++ 0.04 s / 15 MB, with the same output.
- **Emitted C++:** 25493 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
