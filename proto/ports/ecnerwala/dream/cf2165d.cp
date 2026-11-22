// Codeforces 2165 D — Path Split
//
// Partition the sequence `vals[0..n]` into the fewest subsequences whose
// consecutive elements differ by +-1. Minimizing chains equals `n` minus the
// maximum number of directed transitions `i -> j` (`i < j`, `|vals[i] - vals[j]| == 1`)
// where each index has in-degree <= 1 and out-degree <= 1.
//
// Splitting transitions into the two value-parity orientations (`iter in 0..2`,
// alternating whether even or odd values use reversed positions `n - 1 - x`)
// lets us sweep values `v` from `0` to `2 * n` and greedily match each
// occurrence of `v` with the earliest available unmatched occurrence of `v - 1`
// preceding it; unmatched occurrences carry forward to match against `v + 1`.

@!multitest
let n: 1..=500_000 = read()
let vals: [0..n]1..=2 * n = read()
var idx: [0..=2 * n]List<Int> = []
for (i, v) in vals.items: idx[v].push(i)

var ans = n
for iter in 0..2:
    var cur: List<Int> = []
    for v in 0..=2 * n:
        let cnd = if (v & 1) == iter: [n - 1 - x for x in idx[v].rev] else: idx[v]
        let prev = [n - 1 - x for x in cur.rev]
        var (z, stk) = (0, 0)
        var nxt: List<Int> = []
        for a in cnd:
            while z < prev.len and prev[z] < a:
                z += 1
                stk += 1
            if stk > 0:
                stk -= 1
                ans -= 1
            else:
                nxt.push(a)
        cur = nxt
print(ans)
