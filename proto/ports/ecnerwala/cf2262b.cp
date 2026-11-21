// The tournament is decided by its prefix maxima: only a prefix maximum ever
// becomes champion, so the forfeits are the prefix maxima after the first,
// minus the ones a champion eats. A champion that eats a prefix maximum more
// than doubles before it falls, so at most ~30 champions eat; walk only those.
// Removals run backwards as insertions.
@!multitest
let n: 1..=200_000 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
let p: [0..n]I = read()

var total = Fenwick<I, Int>()
var tops = SortedSet<I>()
var eaters: Set<I> = Set()

fn next_top(c: I) -> ?I = if let c1 = c.next: tops.range(c1..).first else: none

fn recheck(c: I):
    let eats = if let q = next_top(c): total.sum(c..q) >= a[q] else: false
    eaters.set(c, on: eats)

fn insert(i: I):
    total.add(i, a[i])
    let before = tops.range(..i).last
    if let c = before, a[c] >= a[i]:
        recheck(c)
        return
    while let q = next_top(i), a[q] <= a[i]:
        let _ = tops.remove(q)
        eaters.remove(q)
    let _ = tops.add(i)
    if let c = before: recheck(c)
    recheck(i)

fn forfeits() -> Int:
    var lost = tops.len - 1
    var from = I.first
    while let c = eaters.range(from..).first:
        var stop = if let q = next_top(c): q.l else: I.end
        loop:
            let s = total.sum(c.l..stop)
            let reach = if let q = tops.bsearch_first(q => a[q] > s): q.l else: I.end
            if reach == stop: break
            stop = reach
        lost -= tops.range(c.l..stop).len - 1
        if let s = stop.right: from = s else: break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)
