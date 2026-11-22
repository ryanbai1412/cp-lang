// apio09p2 Convention Center: the most pairwise-disjoint intervals, and the lexicographically
// smallest set of that size.
// Greedy: after interval i the best next pick is the one with the smallest right end that starts
// after it. Those links form a forest, so "how many fit inside [a, b]" is a depth difference.
let n = read<1..=200_000>()
domain I = 1..=n
let iv: [I](l: Int, r: Int) = read()

let byl = I.range.sorted(by: i => iv[i].l)
let soonest = byl.rev.scan(min(by: i => iv[i].r))
fn first_from(a: Int) -> ?I:
    let k = byl.keys.first_true(k => iv[byl[k]].l >= a)?
    soonest[k]
let t = Rooted.from_parents(I.map(i => first_from(iv[i].r + 1)).collect())
fn fit(a: Int, b: Int) -> Int:
    let u = first_from(a)?? return 0
    if iv[u].r > b:
        return 0
    t.depth(u) - t.depth(t.jump_while(u, v => iv[v].r <= b)!) + 1

var gaps = SortedMap<Int, (end: Int, best: Int)>()
gaps.set(Int.MIN + 1, (end: Int.MAX, best: fit(Int.MIN + 1, Int.MAX)))
var picked = List<I>()
for i in I:
    let (a, g) = gaps.last_le(iv[i].l)!
    if g.end < iv[i].r:
        continue
    let (left, right) = (fit(a, iv[i].l - 1), fit(iv[i].r + 1, g.end))
    if left + 1 + right == g.best:
        picked.push(i)
        gaps.set(a, (end: iv[i].l - 1, best: left))
        gaps.set(iv[i].r + 1, (end: g.end, best: right))
print(picked.len)
print(picked)
