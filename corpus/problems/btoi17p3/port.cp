// Source: Baltic OI 2017 P3 (Railway), https://dmoj.ca/problem/btoi17p3
// Problem: a tree of n <= 1e5 cities with tracks 1..n-1; m <= 5e4 ministers each name s_i >= 2
//   cities (sum S <= 1e5) and want every track on a path between two of them. Print the tracks
//   wanted by at least k ministers, in increasing order.
// Solution: sort a minister's cities by preorder and walk them cyclically: each leg a -> b adds
//   +1 at a and -1 at lca(a, b), so the marks below a track sum to 1 exactly when it is
//   wanted. Summing marks children-first counts the ministers per track. O(S log n + n).
let (n, m, k): (2..=1e5, 1..=5e4, 1..=5e4) = read()
domain City = 1..=n
domain Track = 1..n
let tracks: [Track](u: City, v: City) = read()
let tree = Rooted(Graph.undirected(tracks), City.first!)

// Walking a plan's cities in preorder and back to the start crosses every
// wanted track exactly twice. Mark the start of each leg and cancel at its
// meeting point: the marks below a track then add up to its ministers.
var marks: [City]Int = 0
for _ in 0..m:
    let s: 2..=1e5 = read()
    let plan: [0..s]City = read()
    let tour = plan.sorted(by: tree.pos)
    for i in 0..s:
        marks[tour[i]] += 1
        marks[tree.lca(tour[i], tour[(i + 1) % s])] -= 1
for c in tree.at.rev:  // children before parents
    if let p = tree.parent(c): marks[p] += marks[c]

fn lower(u: City, v: City) -> City = if tree.parent(u) == v: u else: v
let wanted = [t for t in Track if marks[lower(tracks[t].u, tracks[t].v)] >= k]
print(wanted.len)
print(wanted)
