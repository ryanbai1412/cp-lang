// Source: NOI 2010 P4 (Flight Control), https://dmoj.ca/problem/noi10p4
// limits: 600 ms, 512 MB
// Problem: n <= 2000 flights take off one per slot 1..n; flight i must be in a slot <= k_i, and
//   m <= 1e4 rules say a takes off before b. A valid schedule exists. Print one, then for each
//   flight the earliest slot it can have in any valid schedule.
// Solution: tighten deadlines backwards through a topological order, due[a] = min(k_a,
//   due[b] - 1) over rules a -> b; sorting by due is then a schedule. For flight u, mark u and
//   everything that must precede it, then give the unmarked flights the latest slots their
//   deadlines allow, going down from slot n; u takes the highest slot left free. O(n (n + m)).
// original: corpus/submissions/5670273.cpp
// Bounds: n: 1..=2000; m: 0..=10000; k, rules.u, rules.v: 1..=n
let (n, m): (1..=2000, 0..=10000) = read()
domain Plane = 1..=n
domain Rule = 0..m
let k: [Plane]1..=n = read()
let rules: [Rule](u: Plane, v: Plane) = read()
let g = Graph.directed(rules)
let back = g.transpose()

// due[p]: the latest slot p can take, pulled earlier by every plane that must follow it
var due = [Plane] p => k[p]
for p in topo(g)!.rev:
    for a in g[p]: due[p] min= due[a.to] - 1
print(List(Plane).sorted(by: p => due[p]))

// the earliest slot of u: everything that must precede u goes first, the rest as late as
// due allows; u takes the latest slot no other plane is forced into
fn earliest(u: Plane) -> ?Int:
    var first: [Plane]Bool = false
    for p in reachable(back, [u]): first[p] = true
    var due_at: [1..=n]Int = 0
    for p in Plane:
        if not first[p]: due_at[due[p]] += 1
    var waiting = 0
    for slot in (1..=n).rev:
        waiting += due_at[slot]
        if waiting == 0: return slot
        waiting -= 1
    None

print([earliest(u)! for u in Plane])
