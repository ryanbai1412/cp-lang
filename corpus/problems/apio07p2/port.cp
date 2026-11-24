// Source: APIO 2007 P2 (Backup), https://dmoj.ca/problem/apio07p2
// limits: 1000 ms, 32 MB
// Problem: n <= 1e5 offices at sorted positions <= 1e9 on a street; join k <= n / 2 disjoint
//   pairs by cables. Print the least total cable length.
// Solution: an optimal pairing uses k gaps between neighbours, no two adjacent, so choose k
//   such gaps of least sum. Divide and conquer over the gaps: a run keeps, for each choice
//   of which end gaps must stay empty, the convex list of cheapest costs for 0, 1, ... cables.
//   Two halves combine by merging their slopes (a Minkowski sum) with the middle pair of gaps
//   not both used, taking the cheaper option entrywise. O(n log n).
// As it builds today: the halving is a recursive function over list offsets, and the merge
//   of two convex cost lists is written out (R7, R8).
// original: corpus/submissions/5532752.cpp
// Bounds: n: 2..=1e5; k: 1..=n / 2
let n: 2..=1e5 = read()
let k: 1..=n / 2 = read()

type Offices = [n]0..=1e9

fn sorted_offices() -> Offices:
    var at: Offices = 0
    for i in 0..n: at[i] = rand(0, 50)
    at.sort()
    at

let at: Offices = read(gen: sorted_offices)

// Cables join neighbouring offices, and no office takes two: choose k of the
// n - 1 gaps, no two next to each other. For a run of gaps keep four cost
// lists, costs[j] = the cheapest j cables, one per choice of which end gaps
// must stay empty: index 2 * (left end empty) + (right end empty).
type Costs = List<Int>

// costs of j cables split between two runs: both lists are convex, so the
// steps of the answer are the steps of the two, merged in order
fn split(a: Costs, b: Costs) -> Costs:
    var out = [a[0] + b[0]]
    var i = 1
    var j = 1
    while i < a.len or j < b.len:
        if j == b.len or (i < a.len and a[i] - a[i - 1] <= b[j] - b[j - 1]):
            out.push(out.last! + a[i] - a[i - 1])
            i += 1
        else:
            out.push(out.last! + b[j] - b[j - 1])
            j += 1
    out

fn cheaper(a: Costs, b: Costs) -> Costs:
    var out: Costs = []
    for j in 0..max(a.len, b.len):
        if j >= b.len: out.push(a[j])
        elif j >= a.len: out.push(b[j])
        else: out.push(min(a[j], b[j]))
    out

// the two gaps that meet in the middle cannot both carry a cable
fn join(a: List<Costs>, b: List<Costs>) -> List<Costs>:
    [cheaper(split(a[2 * l + 1], b[r]), split(a[2 * l], b[2 + r])) for l in 0..2 for r in 0..2]

fn run(l: Int, r: Int) -> List<Costs>:
    if r - l == 1: return [[0, at[r] - at[l]], [0], [0], [0]]
    let mid = (l + r) / 2
    join(run(l, mid), run(mid, r))

print(run(0, n - 1)[0][k])
