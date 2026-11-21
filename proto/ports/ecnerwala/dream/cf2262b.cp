// Codeforces 2262 B: Culling Game.
// Only a prefix maximum ("top") can ever become champion, and every top after
// the first wins its first fight by forfeit, so the answer is (#tops - 1) minus
// the tops some champion beats instead of forfeiting to. Call a top c an
// eater when it beats the next top q, i.e. a[c..q) sums to at least a[q].
// An eater keeps absorbing tops until the first top above its running sum;
// each absorbed top is larger than everything before it, so the sum doubles
// per top absorbed and the walk takes O(log max a) searches. A forfeit to a
// non-eater changes nothing, so only eaters need walking.
// Removals are answered backwards, as insertions.
@!multitest
let n: 1..=200_000 = read()
domain I = 1..=n
let a: [I]1..=1_000_000_000 = read()
let p: [0..n]I = read()

var total = Fenwick<I, Int>()
var tops: Set<I> = Set()
var eaters: Set<I> = Set()

fn recheck(c: I):
    eaters.set(c, on: tops.first_gt(c) is q and total.sum(c..q) >= a[q])

fn insert(i: I):
    total.add(i, a[i])
    let before = tops.last_lt(i)
    if let c = before, a[c] >= a[i]:
        recheck(c)
        return
    for q in tops.drain(after: i, while: q => a[q] <= a[i]): eaters.remove(q)
    tops.add(i)
    if let c = before: recheck(c)
    recheck(i)

// The cut before the first top whose value exceeds `s`.
fn first_top_above(s: Int) -> Bounds<I> = tops.bsearch_first(q => a[q] > s).map(q => q.l) ?? I.end

// Where eater c's run ends: grow past every top its sum through `stop` beats.
fn reach(c: I, stop: Bounds<I>) -> Bounds<I>:
    let s = first_top_above(total.sum(c.l..stop))
    if s == stop: stop else: reach(c, s)

fn forfeits() -> Int:
    var lost = tops.len - 1
    var from = I.first
    while let c = eaters.first_ge(from):
        let stop = reach(c, tops.first_gt(c).map(q => q.l) ?? I.end)
        lost -= tops.range(c.l..stop).len - 1
        from = stop.right ?? break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)
