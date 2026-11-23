// Source: NOIP 2000 Senior P4 (Square Number Picking), https://dmoj.ca/problem/noip00p4
//   (not public on DMOJ; statement from Luogu P1004)
// Problem: an N x N grid (N <= 9) with positive numbers (<= 30) in some cells, given as
//   lines x y v up to a 0 line. Walk twice from the top-left to the bottom-right corner
//   moving only down or right; a cell's number is taken the first time it is passed.
//   Print the largest total taken.
// Solution: a path is identified by its set of numbered cells: build the distinct sets
//   from each cell to the corner, back to front, as bitmasks over the m numbered cells.
//   Close "is inside some path" under subsets (superset-OR SOS) and take, for every mask,
//   the best path weight inside it (subset-max SOS); the answer is the max over masks s
//   inside a path of weight(s) + best path inside the complement: O(2^m m), m <= 20 here.
// original: corpus/submissions/5067920.cpp
let n: 1..=9 = read()
var cell_id: [0..n, 0..n]Int = -1
var weights: List<Int> = []
fn read_cells():
    while true:
        let x: 0..=9 = read()
        let y: 0..=9 = read()
        let z: 0..=1000 = read()
        if x == 0 and y == 0 and z == 0: break
        cell_id[x - 1, y - 1] = weights.len
        weights.push(z)
read_cells()

let m: 0..=20 = weights.len
domain Item = 0..m
type Mask = Subset<Item>

// dp[r, c]: distinct subsets of weighted cells reachable on a path from (r, c) to (n-1, n-1).
var dp: [0..n, 0..n]List<Mask> = []
for r in (0..n).rev:
    for c in (0..n).rev:
        var cur: List<Mask> = []
        if r == n - 1 and c == n - 1: cur.push(Mask.empty)
        if r + 1 < n:
            for s in dp[r + 1, c]: cur.push(s)
        if c + 1 < n:
            for s in dp[r, c + 1]: cur.push(s)
        let sorted_cur = cur.sorted(by: s => s.idx)
        var uniq: List<Mask> = []
        for s in sorted_cur:
            if uniq.len == 0 or uniq.last! != s: uniq.push(s)
        let id = cell_id[r, c]
        if id != -1:
            let item = Item(id)
            for i in 0..uniq.len: uniq[i] = uniq[i].with(item)
        dp[r, c] = uniq

var reach: [Mask]Bool = false
for s in dp[0, 0]: reach[s] = true

var total: [Mask]Int = 0
var best_sub: [Mask]Int = 0
for s in Mask:
    let w = sum(weights[i.idx] for i in s)
    total[s] = w
    if reach[s]: best_sub[s] = w

// Superset SOS (OR): reach[s] becomes true if any superset of s is a path.
for s in Mask.rev:
    for i in Item:
        if not (i in s):
            if reach[s.with(i)]: reach[s] = true

// Subset SOS (max): best_sub[s] becomes max weight of any path inside s.
for s in Mask:
    for i in s:
        best_sub[s] max= best_sub[s.without(i)]

var ans = 0
for s in Mask:
    if reach[s]: ans max= total[s] + best_sub[Mask.full - s]
print(ans)
