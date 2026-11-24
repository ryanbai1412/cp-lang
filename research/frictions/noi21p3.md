# Friction Report: noi21p3

No friction left: the hand-built condensed adjacency, the in-degree array and the stack-driven
Kahn peel are std `condense` and `topo`. Each component hangs under its last predecessor in
that order, which is unique because the predecessors form a chain. The per-query forward and
backward BFS, with their queues, `head` cursors and visited arrays over every component reset
for each query, are two std `reachable` calls on adjacency lists over the virtual tree's own
domain `vt.K`. The extra edges are a typed `[k](u: V, v: V)` read, and all bounds are typed.

The virtual-tree node of a component is `vt.K.find(x => vt.at[x] == comp[u])!`: a domain
reached through a value has a named domain's members.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++. On the max case the port takes 0.80 s and the C++ 0.70 s, with the same output.
