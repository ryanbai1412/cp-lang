// Source: An Animal Contest 5 P3 (Ski Resort), https://dmoj.ca/problem/aac5p3
// Problem: a tree of n <= 2e5 breakpoints rooted at 1, hills (edges) of difficulty d <= 1e9; each
//   of k <= 2e5 pandas with skill s <= 1e9 starts at 1 and keeps skiing down the child hill with
//   the least |s - d| (ties to the easier) until a leaf. Print how many pandas visit each
//   breakpoint.
// Solution: the skills reaching a node form an interval; sorting its child hills by difficulty
//   splits it at the midpoints between neighbours, a tie going to the easier hill. A DFS hands
//   each child its interval and counts the sorted skills inside by binary search: O((n + k) log).
// original: corpus/submissions/4239862.cpp
let (n, m): (Int, Int) = read()
var adj: List<List<(w: Int, v: Int)>> = [[] for _ in 0..n]
for _ in 0..n - 1:
    let (u, v, w): (Int, Int, Int) = read()
    adj[u - 1].push((w:, v: v - 1))
    adj[v - 1].push((w:, v: u - 1))
var xs: List<Int> = []
for _ in 0..m:
    let x: Int = read()
    xs.push(x)
xs.sort()

// Number of values strictly below x.
fn below(x: Int) -> Int:
    let lo = (0..m).first_true(mid => xs[mid] >= x) ?? m
    return lo

var ans = List.filled(n, 0)

// Values in [l, r) reach u; each child edge keeps the values closest to its weight.
fn dfs(u: Int, p: Int, l: Int, r: Int):
    if l >= r: return
    ans[u] = below(r) - below(l)
    var kids: List<(w: Int, v: Int)> = []
    for e in adj[u]:
        if e.v != p: kids.push(e)
    kids.sort(by: e => e.w)
    for i in 0..kids.len:
        let w = kids[i].w
        let cl = if i > 0: (kids[i - 1].w + w) / 2 + 1 else: l
        let cr = if i + 1 < kids.len: (w + kids[i + 1].w) / 2 + 1 else: r
        dfs(kids[i].v, u, max(cl, l), min(cr, r))

dfs(0, -1, 0, 1000000005)
for x in ans: print(x)
