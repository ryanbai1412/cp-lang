// CCO 2021 P5: given an undirected graph on 1..=n, partition vertices into
// blocks minimizing edge violations. Reduces to dp[i] = min over j in
// (last_bad..=n) of (dp[j] + count of k in i..j with min_nb[k] >= i).
// Maintain -dp in an AddMax<Slot> lazy segment tree so range-min becomes
// range-max and suffix increments become suffix decrements.
// original: corpus/submissions/4554642.cpp
let n: 2..=200000 = read()
let m: 1..=200000 = read()
domain Slot = 0..=n

var min_nb: [0..n]Int = n
for _ in 0..m:
    let a: 1..=n = read()
    let b: 1..=n = read()
    let u = min(a, b) - 1
    let v = max(a, b) - 1
    min_nb[v] min= u

var at_min: [0..n]List<Int> = []
for i in 0..n:
    if min_nb[i] < n: at_min[min_nb[i]].push(i)

var suff: [0..=n]Int = 1000000000
for i in (0..n).rev:
    suff[i] = min(suff[i + 1], min_nb[i])

// Negated AddMax stands in for LazySegTree(AddMin) (not yet in proto/cpc).
const INF = 1000000000
var tree = AddMax<Slot>()
tree.add(Slot.first, Slot(n - 1), -INF)

fn set_dp(idx: Int, val: Int):
    let s = Slot(idx)
    let cur = tree.max(s, s)
    tree.add(s, s, -val - cur)

var j = n
var ans = 0
for i in (1..n).rev:
    while j >= 0 and suff[j] >= i: j -= 1
    for a in at_min[i]:
        tree.add(Slot(a + 1), Slot.last, -1)
    var cur = -tree.max(Slot(j + 1), Slot.last)
    if min_nb[i] >= n:
        cur += 1
        tree.add(Slot(i + 1), Slot.last, -1)
    set_dp(i, cur)
    if i == 1: ans = cur

print(ans)
