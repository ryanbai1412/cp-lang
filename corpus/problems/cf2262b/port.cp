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
var tops = SortedSet<I>()
var eaters = SortedSet<I>()

fn recheck(c: I):
    eaters.set(c, on: tops.range(c.r..).first.map(q => total.sum(c..q) >= a[q]) ?? false)

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
    let nxt = tops.bsearch_first(q => a[q] > total.sum(c.l..stop)).map(q => q.l) ?? I.end
    if nxt == stop: stop else: reach(c, nxt)

fn forfeits() -> Int:
    var lost = tops.len - 1
    var from = I.first
    while let c = eaters.range(from..).first:
        let stop = reach(c, tops.range(c.r..).first.map(q => q.l) ?? I.end)
        lost -= tops.range(c.l..stop).len - 1
        if let s = stop.right: from = s else: break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)
