# Friction Report: noi22p4

No friction left. The v0.2 port's shared `List<List<Int>>` node space offset by |G|, the
`Map<List<Int>, Int>` shape naming, the hand-written sorted merge that cancelled equal child
shapes and the nested backtracking `place` (a closure that compiled to `std::function`) are gone.
Both trees are one per-test `domain Node` forest from `Rooted.from_parents`, `Rooted.shapes`
names every subtree shape, a `Counter<Shape>` cancels equal children, and the leftover children
are matched with `max_matching`. The answer word is `yes(..)` with `io(answers:)`.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's cases pass. On the max case the port takes 0.28 s and the accepted C++
0.25 s, with the same output; explain shows 0 dynamic functions.
