# noip18p3 friction

## `t.child_arcs(v)` yields vertices, not arcs

- **Repro:** `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5),
  (u: 1, v: 2, w: 7)]; let t = Rooted(Graph.undirected(es), root: 0);
  print(t.child_arcs(1).map(a => a.w).collect())` is E0203 "an `Int` has no member `w`"
  (want `[7]`). The built-in `Rooted` yields child vertices typed `Int`; std/tree.cp's
  `child_arcs` and `up_arc` bodies are `todo`.
- **Goal it hurt:** express (spec/lib/graphs-trees.md spells edge data on a tree as
  `up(a.to) + a.w` over `child_arcs`; the old spec.cp did and stopped checking).
- **Workaround:** `t.children(v).map(c => up(c) + t.w(c))`, the parent-edge length.
- **Routed:** compiler worklog.
