# ioi19p6 friction

## `Graph.undirected([es.keys] e => es[e])` as an argument binds `.keys` twice

- **Where:** `corpus/problems/ioi19p6/spec.cp` wants `Graph.undirected([links.keys] e => links[e])`.
- **Repro:** `domain V = 0..3; let es: List<(u: V, v: V)> = [(u: 0, v: 1), (u: 1, v: 2)]; print(Graph.undirected([es.keys] e => es[e]).deg(1))`
  is E0302 "expected `[keys_26](u: V, v: V)`, found `[keys_28](u: V, v: V)`"; binding the array
  first (`let arr = [es.keys] e => es[e]`) prints `2`.
- **Goal it hurts:** express (an inline argument must become a `let`).
- **General fix:** a fresh `.keys` domain is minted once per expression, not once per type query.
- **Severity:** low. **Routed:** compiler worklog.

## `.with_next` of a domain is E0501

- **Where:** pairing consecutive compressed stops, `for (p, q) in Stop.with_next:`.
- **Repro:** `let (domain C, at) = compress([30, 10, 20]); for (p, q) in C.with_next: print(at.get(p))`
  is E0501 "can't handle `.with_next` of a domain as a value yet"; the spec walks
  `Stop.collect().adjacent` (an O(n) copy) instead.
- **Goal it hurts:** express, coherence (a domain is a `Seq`, R04).
- **Severity:** low. **Routed:** compiler worklog.

## C++ over the 32 KB target

- **Where:** `cpl build spec.cp` writes 36795 bytes (W0201, over the 32768-byte target) for a
  60-line program using `SparseTable`, `SortedBag`, `compress`, `Graph` and `dijkstra`.
- **Goal it hurts:** submission.
- **Severity:** medium. **Routed:** perf worklog.

## Known gaps hit, already routed

- `flat_map` loses the element type (`walks.flat_map(w => cut(w, start)).collect()` is a
  `List<Int>`), compiler worklog; the spec uses a comprehension.
- A `@grader fn` has no local stub (tools worklog, apio23p1); the spec reads the sample
  grader's stdin format.
