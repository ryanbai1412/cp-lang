// Source: NOIP 2019 (CSP-S) Day 2 P2 (Partition), https://dmoj.ca/problem/noip19p5
// limits: 2000 ms, 1024 MB
// Problem: a_1..a_n, 2 <= n <= 4e7, 1 <= a_i <= 1e9. Cut it into consecutive blocks whose sums
//   never decrease and print the least sum of squared block sums (up to about 1.6e33). Large
//   tests give a generator instead of the array.
// Solution: an optimal partition makes the last block as short as possible: for the prefix ending
//   at cut c, from[c] is the latest cut j with s[j] - s[from[j]] <= s[c] - s[j], i.e.
//   2 s[j] - s[from[j]] <= s[c]. Those thresholds only matter while increasing, so a monotone
//   deque holds the candidates and its front only moves right. Walk from[] back from the end and
//   sum the squares in I128. O(n).
// input: `n 0` and the array, or `n 1`, `x y z b1 b2 m` and m lines `p l r`:
//   b[i] = (x b[i-1] + y b[i-2] + z) mod 2^30, and a[i] = b[i] mod (r - l + 1) + l for the
//   first line with i <= p.
// Bounds: n: 2..=4e7; kind: 0..=1
// original: corpus/submissions/5148191.cpp

const B = 1 << 30

let (n, kind): (2..=4e7, 0..=1) = read()
domain Pos = 1..=n

var s = Prefix<Pos, Int>.filling()
if kind == 0:
    for i in Pos: s.set(i, read<1..=1e9>())
else:
    let (x, y, z, b1, b2, m): (0..B, 0..B, 0..B, 0..B, 0..B, 1..=1e5) = read()
    let parts: [m](p: Pos, l: 1..=1e9, r: 1..=1e9) = read()
    var (b, after) = (b1, b2)  // b[i], b[i + 1]
    var k = 0
    for i in Pos:
        if i > parts[k].p: k += 1
        s.set(i, b % (parts[k].r - parts[k].l + 1) + parts[k].l)
        (b, after) = (after, (x * after + y * b + z) % B)

var from: [Pos.cuts]?Cut<Pos> = None
fn threshold(j: Cut<Pos>) -> Int = 2 * s.prefix(j) - (s.prefix(from[j]?) ?? 0)

var ready = Deque<Cut<Pos>>.of([Pos.cuts.first!])
for i in Pos:
    let c = i.r
    while ready.len >= 2 and threshold(ready.get(1)) <= s.prefix(c): ready.pop_front()
    from[c] = ready.first
    while threshold(ready.last!) >= threshold(c): ready.pop()
    ready.push(c)

var total: I128 = 0
for c in iterate(Some(Pos.cuts.last!), c => from[c]):
    if let j = from[c]: total += I128(s.prefix(c) - s.prefix(j)) ** 2
print(total)
