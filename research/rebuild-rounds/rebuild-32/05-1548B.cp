// cf1548b Integers Have Friends: a friend group is a run whose adjacent differences
// have gcd >= 2; the longest such run of differences, plus one.

@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=1e18>()
let d = [0..n - 1] i => abs(a[i + 1] - a[i])
// found late: `Window(gcd)` fails in std.window, so the gcds of every piece ending at i
// are kept by hand, one entry per distinct gcd with its longest piece.
var best = 0
var ends = List<(g: Int, len: Int)>()
var next = List<(g: Int, len: Int)>()
for i in d.keys:
    next.clear()
    next.push((g: d[i], len: 1))
    for e in ends:
        let g = gcd(e.g, d[i])
        if g == next.last!.g: next[next.len - 1] = (g: g, len: e.len + 1)
        else: next.push((g: g, len: e.len + 1))
    swap(mut ends, mut next)
    for e in ends:
        if e.g != 1: best max= e.len
print(best + 1)
