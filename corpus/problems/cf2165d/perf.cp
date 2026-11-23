// Codeforces 2165 D, tuned: port.cp's two greedy sweeps with the carried and
// unmatched positions in two buffers allocated once and swapped, and the
// reflections read through the index (`n - 1 - xs[len - 1 - k]`) instead of
// building a reflected copy of each list, as orig.cpp reuses `cur` and `nxt`.
@!multitest
let n: 1..=1_000_000 = read()
let vals: [0..n]1..=2 * n = read()
var by_val: [0..=2 * n]List<Int> = []
for (i, v) in vals.items: by_val[v].push(i)

var ans = n
for flip in 0..2:
    var carried: List<Int> = []
    var unmatched: List<Int> = []
    carried.reserve(n)
    unmatched.reserve(n)
    for v in 0..=2 * n:
        let here = by_val[v]
        let flipped = v % 2 == flip
        let m = carried.len
        var (ptr, available) = (0, 0)
        for k in 0..here.len:
            let pos = if flipped: n - 1 - here[here.len - 1 - k] else: here[k]
            while ptr < m and n - 1 - carried[m - 1 - ptr] < pos:
                ptr += 1
                available += 1
            if available > 0:
                available -= 1
                ans -= 1
            else:
                unmatched.push(pos)
        swap(mut carried, mut unmatched)
        unmatched.clear()
print(ans)
