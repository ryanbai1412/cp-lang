// CF 1000C: n segments [l, r] with 0 <= l <= r <= 1e18; for each k in 1..=n, count the
// integer points covered by exactly k segments. Compress l and r + 1, sweep the coverage.
let n = read<1..=2e5>()
let segs = read<[n]Range<0..=1e18>>()
let (domain X, xs) = compress(chain(segs.map(s => Int(s.first!)), segs.map(s => Int(s.last!) + 1)))
var delta: [X]Int = 0
for s in segs:
    delta[xs.at(Int(s.first!))] += 1
    delta[xs.at(Int(s.last!) + 1)] -= 1
let cover = delta.scan(+)
var cnt: [0..=n]Int = 0
for (x, nx) in X.range.adjacent:
    cnt[cover[x]] += xs.get(nx) - xs.get(x)
print(cnt[1..])
