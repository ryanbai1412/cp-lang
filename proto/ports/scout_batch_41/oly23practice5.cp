// OLY 2023 Practice 5: minimum edits to turn a[0..n] into an alternating
// sequence (x, y, x, y, ...) with x != y, accounting for the L / 2 penalty on
// each maximal reversed alternating run of length L.
// original: corpus/submissions/6415646.cpp
@multitest
let n: 1..=300_000 = read()
let a: [0..n](1..=n) = read()

var pen = HashMap<(Int, Int), Int>()
for i in 0..(n - 1):
    if i > 0 and a[i + 1] == a[i - 1]: continue
    var j = i + 2
    while j < n and a[j] == a[i + (j - i) % 2]: j += 1
    let (ca, cb) = if i % 2 == 0: (a[i + 1], a[i]) else: (a[i], a[i + 1])
    pen[(ca, cb)] += (j - i) / 2

var even: [0..=n]Int = 0
var odd: [0..=n]Int = 0
for (i, x) in a.items:
    if i % 2 == 0: even[x] += 1
    else: odd[x] += 1

var ans = n
for ((x, y), extra) in pen.items if x != y:
    ans min= n - even[x] - odd[y] + extra

let by_odd = (0..=n).sorted(by: y => -odd[y])
for x in 0..=n:
    if let y = first(y for y in by_odd if y != x and (x, y) not in pen):
        ans min= n - even[x] - odd[y]
print(ans)
