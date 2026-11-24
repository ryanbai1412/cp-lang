# Friction Report: noi10p3

## 1. `Graph.directed` takes only an array of edges
- **Where:** `domain Arc = 0..dual.len; Graph.directed([Arc] a => dual[a.ord])`.
- **What felt wrong:** the four kinds of dual arc are pushed onto a list as the segments are
  read, but `Graph.directed` wants a domain-indexed array, so the list is copied into one.
- **Goal it hurts:** one spelling per idea; building graphs.
- **Ideal code:** `Graph.directed(dual)` with `dual: List<(u: Face, v: Face, w: Int)>`.
- **General fix:** let `Graph.directed` / `Graph.undirected` take any `Seq` of edge records.
- **Severity:** low.

## 2. Rethought: the planar dual instead of an 8-direction walk
The v0.2 port walked junction-direction states and turned through 8 directions; the dual has
one vertex per cell and one arc per segment flow, read straight from the four flow grids.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 500) 0.06 s vs C++ 0.41 s.
