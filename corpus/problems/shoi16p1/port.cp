// Source: JLOI/SHOI 2016 (Scout Guards), https://dmoj.ca/problem/shoi16p1
//   (not public on DMOJ; statement from Luogu P3267)
// limits: 2000 ms, 250 MB
// Problem: a tree on N <= 5e5 nodes; a guard at v costs W_v <= 1000 and watches every node
//   within distance D <= 20. Given M marked nodes, print the cheapest set of guards
//   watching all of them.
// Solution: tree DP on a signed reach r in -D..=D. best[v, r], r >= 0: v's subtree is
//   watched and a guard in it still sees r levels above v; r < 0: all is watched except the
//   top -r levels, left to a guard outside. best[v, r] is a suffix min over r. Reach D puts a
//   guard on v; 0 <= r < D lets one child bring reach r + 1 while the others need only -r;
//   r < 0 asks r + 1 of every child; an unmarked v may leave itself open at r = 0. O(N D).
// original: corpus/submissions/5445691.cpp
let n: 1..=5e5 = read()
let k: 1..=20 = read()
domain V = 1..=n
domain Reach = -k..=k
let cost: [V]1..=1000 = read()
let m: 1..=n = read()
let must: [m]V = read()
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)

var marked: [V]Bool = false
for v in must: marked[v] = true

var best: [V, Reach]Int = 0
for v in t.order.rev:
    for r in Reach.rev:
        var b = if r == Reach.last!: cost[v] + sum(best[c, Reach.first!] for c in t.children(v))
            elif r.label >= 0:
                let carry = min(best[c, r.next!] - best[c, Reach(-r.label)] for c in t.children(v))
                if let extra = carry: sum(best[c, Reach(-r.label)] for c in t.children(v)) + extra
                else: best[v, r.next!]
            else: sum(best[c, r.next!] for c in t.children(v))
        if let up = r.next: b min= best[v, up]
        if r.label == 0 and not marked[v]: b min= sum(best[c, Reach(0)] for c in t.children(v))
        best[v, r] = b
print(best[t.root, Reach(0)])
