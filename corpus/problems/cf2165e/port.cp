// Source: Codeforces 2165E (Rainbow Branch), https://codeforces.com/contest/2165/problem/E
// Problem: a tree on n vertices (multitest, sum n <= 3e5). Colour its edges with exactly k
//   colours, minimising the most colours on any path; print that minimum for every k = 1..n-1.
// Solution: find f(c), the most colours usable with at most c per path; the answer for k is
//   the least c with f(c) >= k. f(1) = 1; an odd budget 2r + 1 gives each edge of the r outer
//   leaf layers its own colour, so it adds the size of the next layer; an even budget adds a
//   star of deg - 1 fresh colours at the highest-degree vertex left. Peel layers keeping the
//   degrees in a SortedBag, O(n log n).

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())

var deg = [V] v => g[v].len
var degs = SortedBag<Int>.of([deg[v] for v in V])
var leaves = [v for v in V if deg[v] == 1]
var alive: [V]Bool = true

fn peel_layer():
    var next_leaves: List<V> = []
    for v in leaves:
        alive[v] = false
        degs.remove(deg[v])
        for nb in g[v] if alive[nb]:
            degs.remove(deg[nb])
            deg[nb] -= 1
            degs.add(deg[nb])
            if deg[nb] == 1: next_leaves.push(nb)
    leaves = next_leaves

var edge_cnt = 1
var colors = 2
var ans = [1]

while ans.len < n - 1:
    let max_deg = degs.last!
    if colors % 2 == 0:
        edge_cnt += max_deg - 1
    else:
        edge_cnt += leaves.len - (max_deg - 1)
        peel_layer()
    while ans.len < edge_cnt: ans.push(colors)
    colors += 1

print(ans)
