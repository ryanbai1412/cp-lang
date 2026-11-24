# Friction Report: noi23p4

No friction left. The v0.2 port's eleven `g_*` globals, two parallel lists as a hand-written
binary heap, the save/load recursion that copied a subtree's distances into a buffer and back,
and the `INF` sentinel are gone. Distances are a `RollbackArray<City, ?Int>` (absent is
unreached; `mark` / `undo_to` restore a subtree on return), the Dijkstra frontier is a std
`MinHeap`, the tree's roads up and the extra roads are one `Graph.directed`, and the running sum
and count of reached cities ride down the DFS as arguments, so returning restores them for free.
Reads are typed by `domain City` and the statement's bounds.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's cases pass. On the max case the port takes 0.16 s and the accepted C++
0.19 s, with the same output.
