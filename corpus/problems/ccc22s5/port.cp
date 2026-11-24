// Source: CCC 2022 Senior S5 (Good Influencers), https://dmoj.ca/problem/ccc22s5
// Problem: a tree of n <= 2e5 students, each initially Y or N (at least one of each), with costs
//   c_i <= 1000. Paying a Y student c_i turns all their friends Y. Print the least total cost
//   to make every student Y.
// Solution: tree DP from the leaves, three ?Int costs per subtree (None if impossible, so lifted + and
//   min skip it): push[u], u
//   pays (an N u first needs one child that pays); done[u], the subtree ends all Y without the
//   parent; need[u], the same once the parent's seminar has reached u. Each child adds the
//   cheapest state its parent allows, and an N vertex upgrades the one child whose push costs
//   least extra. O(n); the answer is min(done, push) at the root.
// original: corpus/submissions/4371349.cpp
// Bounds: n: 2..=2e5; edges.u, edges.v: 1..=n; cost: 1..=1000
let n: 2..=2e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let initial: [V]Char = read()
let cost: [V]1..=1000 = read()
let tree = Rooted(Graph.undirected(edges), V.first!)

var push: [V]?Int = None  // u pays
var done: [V]?Int = None  // the subtree ends all Y on its own
var need: [V]?Int = None  // the same once the parent has paid
for u in tree.at.rev:
    var settled: ?Int = 0  // every child ends Y on its own
    var helped: ?Int = 0   // every child, once u has paid
    for c in tree.children(u):
        settled = settled + min(done[c], push[c])
        helped = helped + min(need[c], min(done[c], push[c]))
    var lit_push = if initial[u] == 'Y': helped else: None
    var lit_done = if initial[u] == 'Y': settled else: None
    if initial[u] == 'N':  // one child pays first and turns u Y
        for c in tree.children(u):
            lit_push min= helped - min(need[c], min(done[c], push[c])) + push[c]
            lit_done min= settled - min(done[c], push[c]) + push[c]
    push[u] = cost[u] + lit_push
    done[u] = min(lit_done, push[u])
    need[u] = min(done[u], min(settled, cost[u] + helped))
print(min(done[V.first!], push[V.first!])!)
