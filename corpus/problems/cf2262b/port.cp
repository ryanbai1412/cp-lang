// Source: Codeforces 2262B (Culling Game), https://codeforces.com/contest/2262/problem/B
// Problem: n <= 2e5 sorcerers (multitest, sum n <= 2e5), skills a_i <= 1e9. The leftmost is
//   champion and meets the rest left to right: if his skill s < x he forfeits and x takes
//   over, else he wins and s += x. For each i = 0..n-1, after removing sorcerers p_1..p_i
//   (p a permutation), print the number of forfeits.
// Solution: only a prefix maximum (a top) can become champion, and the answer is (#tops - 1)
//   minus the tops some champion beats. An eater c beats the next top; it absorbs tops until one
//   exceeds its running sum, which at least doubles per top, so its walk takes O(log max a)
//   searches. Answer removals backwards as insertions, with a Fenwick tree of skills and sorted
//   sets of the tops and the eaters.
@!multitest
let n: 1..=200_000 = read()
domain I = 1..=n
let a: [I]1..=1_000_000_000 = read()
let p: [0..n]I = read()

var total = Fenwick<I, Int>()
var tops = SortedSet<I>()
var eaters = SortedSet<I>()

fn recheck(c: I):
    eaters.set(c, on: tops.range(c.r..).first.map(q => total.fold(c..q) >= a[q]) ?? false)

fn insert(i: I):
    total.add(i, a[i])
    let before = tops.range(..i).last
    if let c = before and a[c] >= a[i]:
        recheck(c)
        return
    for q in tops.drain(after: i, while: q => a[q] <= a[i]): eaters.remove(q)
    tops.add(i)
    if let c = before: recheck(c)
    recheck(i)

// Where eater c's run ends: grow past every top its sum through `stop` beats.
fn reach(c: I, stop: I.bounds) -> I.bounds:
    let nxt = tops.first_true(q => a[q] > total.fold(c.l..stop)).map(q => q.l) ?? I.end
    if nxt == stop: stop else: reach(c, nxt)

fn forfeits() -> Int:
    var lost = tops.len - 1
    var from = I.first
    while let c = eaters.range(from!..).first:
        let stop = reach(c, tops.range(c.r..).first.map(q => q.l) ?? I.end)
        lost -= tops.range(c.l..stop).len - 1
        if let s = stop.right: from = s else: break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)
