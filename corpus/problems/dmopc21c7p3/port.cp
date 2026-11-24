// Source: DMOPC '21 Contest 7 P3 (Whack-an-Ishigami), https://dmoj.ca/problem/dmopc21c7p3
// limits: 2000 ms, 256 MB
// Problem: N <= 1e6 figures, some up (k_i = 1), and M <= 1e6 distinct links a -> b (a != b):
//   flipping a also flips b, in a chain reaction; an endless reaction resets the game. A whack
//   flips one figure. Print the fewest whacks that leave every figure down, or -1.
// Solution: a figure is flipped once per walk from a whacked figure to it, so on a DAG
//   flips(v) = whacked(v) + sum of flips over its in-links, and flips must equal k: v is
//   whacked iff k_v differs from the XOR of k over its in-links, a unique choice. Every whack
//   so chosen is a lit figure or reachable from one, and every lit figure needs a whack that
//   reaches it, so the answer is -1 iff a cycle is reachable from a lit figure: `topo` of
//   the links leaving figures `reachable` from the lit ones. O(N + M).
// original: corpus/submissions/4453037.cpp
// forward adjacency as linked lists: head[u] -> edge, nxt[edge]
// DFS colours over what the lit lamps reach: 1 = on the stack, 2 = done
// Bounds: n: 1..=1e6; m: 0..=1e6; up: 0..=1; links.u, links.v: 1..=n

let n: 1..=1e6 = read()
let m: 0..=1e6 = read()
domain Figure = 1..=n
let up: [Figure]0..=1 = read()
let links: [m](u: Figure, v: Figure) = read()

// fed[v]: the XOR of `up` over v's in-links, the flips v gets from figures above it
var fed: [Figure]0..=1 = 0
for w in links: fed[w.v] ^= up[w.u]

let g = Graph.directed(links)
var lit_reach: [Figure]Bool = false
for v in reachable(g, [v for v in Figure if up[v] == 1]): lit_reach[v] = true
let live = Graph.directed([(w.u, w.v) for w in links if lit_reach[w.u]])

if topo(live) == None: print(-1)
else: print(len([v for v in Figure if fed[v] != up[v]]))
