// btoi20p3 Joker: does closing streets l..=r still leave an odd cycle among the rest?
// last[l], the largest r that still does, never decreases as l grows, so divide and conquer
// over l with a rollback parity DSU fills it with O(m log m) unions.
let (n, m, q) = read<(1..=200_000, 1..=200_000, 1..=200_000)>()
domain V = 1..=n
let street = read<[m](V, V)>()
var d = ParityDsu<V>()
fn add(e: Int):
    d.unite(street[e].0, street[e].1)
var last: [0..m]Int = -1

// Answers for l..=r lie in lo..=hi; every street below l or above hi is already in `d`.
fn fill(l: Int, r: Int, lo: Int, hi: Int):
    if l > r:
        return
    let mid = (l + r) / 2
    let base = d.mark()
    for e in l..mid:
        add(e)
    var cut = hi
    while d.consistent:
        add(cut)
        cut -= 1
    last[mid] = cut
    d.undo_to(base)
    for e in l..=mid:
        add(e)
    fill(mid + 1, r, cut, hi)
    d.undo_to(base)
    for e in cut + 1..=hi:
        add(e)
    fill(l, mid - 1, lo, cut)
    d.undo_to(base)

for e in 0..m:
    add(e)
let odd = not d.consistent
d = ParityDsu<V>()
if odd:
    fill(0, m - 1, 0, m - 1)
for (l, r) in read<[q](1..=m, 1..=m)>():
    print(yes(odd and r - 1 <= last[l - 1]))
