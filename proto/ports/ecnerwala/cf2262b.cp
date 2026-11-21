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
var tops = Fenwick<I, Int>()
var top_at: Set<I> = Set()
var top_of: Map<Int, I> = Map()
var eaters: Set<I> = Set()

fn next_top(c: I) -> Bounds<I> = if let q = top_at.first_gt(c): q.l else: I.end

fn recheck(c: I):
    eaters.remove(c)
    if let q = top_at.first_gt(c), total.sum(c..q) >= a[q]: eaters.add(c)

fn insert(i: I):
    total.add(i, a[i])
    let before = top_at.last_lt(i)
    if let c = before, a[c] >= a[i]:
        recheck(c)
        return
    while let q = top_at.first_gt(i), a[q] <= a[i]:
        top_at.remove(q)
        top_of.remove(a[q])
        eaters.remove(q)
        tops.add(q, -1)
    top_at.add(i)
    top_of[a[i]] = i
    tops.add(i, 1)
    if let c = before: recheck(c)
    recheck(i)

fn forfeits() -> Int:
    var lost = top_at.len - 1
    var from = I.first
    while let c = eaters.first_ge(from):
        var stop = next_top(c)
        loop:
            let reach = if let (_, q) = top_of.first_gt(total.sum(c.l..stop)): q.l else: I.end
            if reach == stop: break
            stop = reach
        lost -= tops.sum(c.l..stop) - 1
        if let s = stop.right: from = s else: break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)
