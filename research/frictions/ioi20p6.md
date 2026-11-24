# Friction report: ioi20p6

## Dream

```cp
@grader fn label(tree: Tree<Station>) -> [Station]Label =
    tree.euler(enter: s => tree.depth(s) % 2 == 0).inverse()
@grader fn find_next_station(s: Label, t: Label, near: List<Label>) -> Label =
    if s < near[0]: near.take(near.len - (s != Label.first!)).find(c => s < t <= c) ?? near.last!
    else: near.drop(1).rev().find(c => c <= t < s) ?? near[0]
```

## The grader lives in the solution

The task is two grader functions run in separate processes: `label` sees the tree, and
`find_next_station` sees only labels. The port runs both in one program and does the
grader's work (reading queries, sorting neighbour labels, mapping a label back to a
station). A two-phase `@grader` entry would keep the port to the two functions and would
check that the second one reads nothing but its arguments.

## Numbering on entry or exit is a hand walk

`walk` pushes a station before or after its children by depth parity. A std Euler order with
a per-vertex choice of entry or exit time (`tree.euler(enter: ..)`) would name it and drop
the recursive fn and the `order` list.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (`orig.cpp`, the AC
grader functions plus a stdin `main`), and the max case (n = 1000, q = 1e5) is
byte-identical to the C++ and runs in 0.01 s.
