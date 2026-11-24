// Codeforces 2183 E, tuned: port.cp's DP with the rows over `0..=m`, as
// orig.cpp stores them, so `x` is its own index, the sweep `unchecked:`, and
// the divisor loop innermost, as orig.cpp orders it, stepping both indices
// by addition instead of multiplying them out.
type Z = Mod<998_244_353>

@!multitest
let (n, m): (2..=3000, 2..=3000) = read()
let a: [0..n]0..=m = read()
var ways: [0..=m]Z = 0
var next: [0..=m]Z = 0
if a[0] in 0..=1: ways[1] = 1
for i in 1..n:
    next.fill(0)
    unchecked:
        for j in 1..m:
            var (from, to) = (j, j + 1)
            while to <= m:
                next[to] += ways[from]
                (from, to) = (from + j, to + j + 1)
    if a[i] != 0:
        for x in 1..=m if x != a[i]: next[x] = 0
    swap(mut ways, mut next)
print(ways.sum())
