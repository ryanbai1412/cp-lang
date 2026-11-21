// The tournament is decided by its prefix maxima: only a prefix maximum ever becomes champion,
// so the forfeits are the prefix maxima after the first, minus the ones a champion eats. A
// champion that eats a prefix maximum more than doubles before it falls, so at most ~30
// champions eat; walk only those. Removals run backwards as insertions.
@!multitest
let n: 1..=200_000 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
let p: [0..n]I = read()

var total = Fenwick<I, Int>()
var tops: Set<I> = Set()        // increasing in position and in value: one ordered set answers both
var eaters: Set<I> = Set()

fn recheck(c: I):
    eaters[c] = tops.first_gt(c) is q and total.sum(c..q) >= a[q]

fn insert(i: I):
    total.add(i, a[i])
    let before = tops.last_lt(i)
    if before is c and a[c] >= a[i]:
        recheck(c)
        return
    for q in tops.drain(after: i, while: q => a[q] <= a[i]): eaters[q] = false
    tops.add(i)
    if before is c: recheck(c)
    recheck(i)

fn forfeits() -> Int:
    var lost = tops.len - 1
    var from = I.first
    while eaters.first_ge(from) is c:
        let stop = fixpoint(tops.first_gt(c)?.l ?? I.end,
            s => tops.first(q => a[q] > total.sum(c.l..s))?.l ?? I.end)
        lost -= tops.count(c..stop) - 1
        from = stop.right ?? break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)
