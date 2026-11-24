// Source: An Animal Contest 5 P3 (Ski Resort), https://dmoj.ca/problem/aac5p3
// Problem: a tree of n <= 2e5 breakpoints rooted at 1, hills (edges) of difficulty d <= 1e9; each
//   of k <= 2e5 pandas with skill s <= 1e9 starts at 1 and keeps skiing down the child hill with
//   the least |s - d| (ties to the easier) until a leaf. Print how many pandas visit each
//   breakpoint.
// Solution: the skills reaching a node form an interval; sorting its child hills by difficulty
//   splits it at the midpoints between neighbours, a tie going to the easier hill. Walking the
//   tree parents first hands each child its interval, and `count_in` counts the sorted skills
//   inside: O((n + k) log).
// original: corpus/submissions/4239862.cpp
// Bounds: n, k: 1..=2e5; hills.u, hills.v: 1..=n; hills.d: 1..=1e9
let (n, k): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let hills: [n - 1](u: V, v: V, d: 1..=1e9) = read()
let skills = read<[k]1..=1e9>().sorted()
let tree = Rooted(Graph.undirected([(u, v) for (u, v, _) in hills]), V.first!)
var climb: [V]Int = 0  // the difficulty of the hill down into v
for (u, v, d) in hills:
    climb[tree.lower(u, v)] = d

// the skills that reach v, a half-open interval
var reach: [V]Range<Int> = 0..0
reach[V.first!] = 0..1e9 + 1
for v in tree.order:
    let down = tree.children(v).collect().sorted(by: c => climb[c])
    for (i, c) in down.items:
        // a skill goes to the closer of two neighbouring hills, the easier one on a tie
        let lo = if i > 0: (climb[down[i - 1]] + climb[c]) / 2 + 1 else: reach[v].l
        let hi = if i + 1 < down.len: (climb[c] + climb[down[i + 1]]) / 2 + 1 else: reach[v].r
        reach[c] = max(lo, reach[v].l)..max(lo, min(hi, reach[v].r))
for v in V:
    print(skills.count_in(reach[v]))
