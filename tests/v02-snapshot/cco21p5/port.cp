// Source: CCO 2021 P5, Bread First Search, https://dmoj.ca/problem/cco21p5
// Problem: an undirected simple graph with n <= 2e5 towns and m <= 2e5 roads. Print the least
//   number of new roads that make 1, 2, ..., n a valid BFS order (from town 1).
// Solution: BFS layers are contiguous blocks; vertex v (not 1) needs its smallest earlier
//   neighbour `min_nb[v]` in the previous block, or one new road. dp[i] = least roads for towns
//   i.. when a block starts at i; the next block start s must pass every town whose smallest
//   earlier neighbour precedes i (pointer j over suffix minima). A town in block [i, s) whose
//   smallest earlier neighbour lies in its own block, or who has none, costs one road, so scan
//   i down and add 1 to the suffix of s it affects in a lazy SegTree of ?Int minimums (empty
//   slots are None, so no INF sentinel); dp[i] is a range min. O((n + m) log n).
// original: corpus/submissions/4554642.cpp

let (n, m): (2..=200_000, 1..=200_000) = read()
domain Slot = 0..=n

var min_nb: [0..n]Int = n
for _ in 0..m:
    let (a, b): (1..=n, 1..=n) = read()
    let (u, v) = (min(a, b) - 1, max(a, b) - 1)
    min_nb[v] min= u

var at_min: [0..n]List<Int> = []
for i in 0..n:
    if min_nb[i] < n: at_min[min_nb[i]].push(i)

var suff: [0..=n]Int = n
for i in (0..n).rev: suff[i] = min(suff[i + 1], min_nb[i])

fn opt_min(a: ?Int, b: ?Int) -> ?Int:
    var out = a
    out min= b
    out

let add_min = Action<?Int, Int>(
    0,
    (f, g) => f + g,
    (f, x, _) => x + f,
    over: Monoid<?Int>(None, opt_min),
)
let empty: [Slot]?Int = None
var tree = SegTree(add_min.over, empty, act: add_min)
tree.set(Slot.last!, Some(0))

var j = n
var ans = 0
for i in (1..n).rev:
    while j >= 0 and suff[j] >= i: j -= 1
    for a in at_min[i]: tree.apply(Slot(a + 1)..=Slot.last!, 1)
    var cur = tree.fold(Slot(j + 1)..=Slot.last!) ?? 0
    if min_nb[i] >= n:
        cur += 1
        tree.apply(Slot(i + 1)..=Slot.last!, 1)
    tree.set(Slot(i), Some(cur))
    if i == 1: ans = cur

print(ans)
