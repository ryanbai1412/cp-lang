// Codeforces 2147 F, tuned: port.cp's cut cover with orig.cpp's tree. A cover
// count stays on the nodes of the range it was added to (no push down), and
// each node keeps the first and last uncovered cut below it and their gap sum,
// all in flat arrays over a power-of-two layout.

@!multitest
let n: 2..=100_000 = read()
let q: 1..=100_000 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]Cut = read()
var s: [Item]Cut = read()

var size = 1
while size <= n: size *= 2
var held: [0..2 * size]0..=n = 0
var first: [0..2 * size]-1..=n = -1
var last: [0..2 * size]-1..=n = -1
var gaps: [0..2 * size]Int = 0

fn pull(a: Int):
    if held[a] > 0:
        first[a] = -1
        last[a] = -1
        gaps[a] = 0
    elif a >= size:
        let c = if a - size <= n: a - size else: -1
        first[a] = c
        last[a] = c
        gaps[a] = 0
    else:
        let (l, r) = (2 * a, 2 * a + 1)
        var g = gaps[l] + gaps[r]
        if last[l] != -1 and first[r] != -1: g += choose2(first[r] - last[l])
        gaps[a] = g
        first[a] = if first[l] != -1: first[l] else: first[r]
        last[a] = if last[r] != -1: last[r] else: last[l]

fn cover(i: Item, d: Int):
    let (lo, hi) = minmax(p[i], s[i])
    if lo == hi: return
    var (l, r) = (lo + size, hi + size)
    while l < r:
        if l % 2 == 1:
            held[l] += d
            pull(l)
            l += 1
        if r % 2 == 1:
            r -= 1
            held[r] += d
            pull(r)
        l /= 2
        r /= 2
    for end in [lo + size, hi + size - 1]:
        var a = end / 2
        while a >= 1:
            pull(a)
            a /= 2

for a in (1..2 * size).rev: pull(a)
for i in Item: cover(i, 1)
repeat q:
    let (t, i, j): (1..=2, Item, Item) = read()
    for x in [i, j]: cover(x, -1)
    if t == 1: p.swap(i, j) else: s.swap(i, j)
    for x in [i, j]: cover(x, 1)
    print(choose2(n + 1) + gaps[1])
