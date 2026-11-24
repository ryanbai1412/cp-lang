// Source: OLY 2023 Practice 5, https://dmoj.ca/problem/oly23practice5
// Problem: change as few elements as possible of an array a (n <= 3e5, values in 1..=n)
//   so that it alternates between two distinct values x and y, with x on even positions
//   and y on odd ones. Multitest.
// Solution: keeping x on even and y on odd positions costs n - even[x] - odd[y], plus a
//   penalty where a run of the pair alternates the wrong way. Only the O(n) pairs that
//   appear next to each other carry a penalty: try those, and for every other x the
//   best penalty-free y in order of odd count.
// original: corpus/submissions/6415646.cpp
// Bounds: n: 1..=300_000; a: 1..=n
@!multitest
let n: 1..=300_000 = read()
let a: [0..n]1..=n = read()

// pen[(x, y)]: the extra changes that keeping x on even and y on odd
// positions costs, from maximal runs alternating the pair the wrong way round
var pen = Counter<(Int, Int)>()
for i in 0..n - 1:
    if i > 0 and a[i + 1] == a[i - 1]: continue
    var j = i + 2
    while j < n and a[j] == a[j - 2]: j += 1
    let pair = if i % 2 == 0: (a[i + 1], a[i]) else: (a[i], a[i + 1])
    pen.add(pair, (j - i) / 2)

// how often each value already sits on an even or an odd position;
// 0 never appears, which is the choice of a fresh value
var even: [0..=n]Int = 0
var odd: [0..=n]Int = 0
for i in 0..n:
    if i % 2 == 0: even[a[i]] += 1
    else: odd[a[i]] += 1

var ans = min(
    n,
    pen.items.filter(((x, y), extra) => x != y).map(
        ((x, y), extra) => n - even[x] - odd[y] + extra
    ).min() ?? n,
)

// every other pair costs no penalty: for each x, the best such y
let by_odd = (0..=n).collect().sorted(by: y => desc(odd[y]))
for x in 0..=n:
    for y in by_odd:
        if y != x and (x, y) not in pen:
            ans min= n - even[x] - odd[y]
            break
print(ans)
