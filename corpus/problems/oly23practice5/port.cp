// Source: OLY 2023 Practice 5, https://dmoj.ca/problem/oly23practice5
// Problem: change as few elements as possible of an array a (n <= 3e5, values in 0..=n)
//   so that it alternates between two distinct values x and y, with x on even positions
//   and y on odd ones. Multitest.
// Solution: keeping x on even and y on odd positions costs n - even[x] - odd[y], plus a
//   penalty where a run of the pair alternates the wrong way. Only the O(n) pairs that
//   appear next to each other carry a penalty: try those, and for every other x the
//   best penalty-free y in order of odd count.
// original: corpus/submissions/6415646.cpp
// Bounds: n: 1..=300_000
@!multitest
let n: 1..=300_000 = read()
domain I = 0..n
let a: [I]Int = read()

var pen = Map<(Int, Int), Int>()
for i in 0..(n - 1):
    if i > 0 and a[I.nth(i + 1)] == a[I.nth(i - 1)]: continue
    var j = i + 2
    while j < n and a[I.nth(j)] == a[I.nth(i + (j - i) % 2)]:
        j += 1
    var ca = a[I.nth(i)]
    var cb = a[I.nth(i + 1)]
    if i % 2 == 0:
        swap(mut ca, mut cb)
    let k = (ca, cb)
    let prev = pen.get(k) ?? 0
    pen[k] = prev + (j - i) / 2

var even: [0..=n]Int = 0
var odd: [0..=n]Int = 0
for i in 0..n:
    let x = a[I.nth(i)]
    if i % 2 == 0: even[x] += 1
    else: odd[x] += 1

var ans = n
for (k, extra) in pen:
    let x = k.0
    let y = k.1
    if x != y:
        ans min= n - even[x] - odd[y] + extra

var by_odd: List<(Int, Int)> = [(-odd[y], y) for y in 0..=n]
by_odd.sort()

for x in 0..=n:
    for p in by_odd:
        let y = p.1
        if y != x and pen.get((x, y)) == None:
            ans min= n - even[x] - odd[y]
            break
print(ans)
