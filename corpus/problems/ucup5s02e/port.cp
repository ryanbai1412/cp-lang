// Universal Cup 5 Stage 2 (EPS) E — Common Prefix Cuts
//
// A cut after 0-based position `i` (`0 <= i < n - 1`) is good when two of the
// three rankings `rank[z]`, `rank[y]` contain the exact same set of contestants
// in their first `i + 1` places; the answer is `1 + good.count(b => b)`.
//
// Inverting `rank[y]` (`place = rank[y].inverse()`) maps each contestant to
// their 0-based position in `rank[y]`. Because both rankings are bijections on
// `P`, the first `i + 1` elements of `rank[z]` occupy `{0, ..., i}` in `rank[y]`
// iff their maximum position `reach[i] = max_{j <= i} place[rank[z][j]]` equals `i`.

@!multitest
let n: 1..=200_000 = read()
domain P = 1..=n
let rank: [0..3][0..n]P = read()

var good: [0..n - 1]Bool = false
for z in 0..3:
    for y in z + 1..3:
        let place = rank[y].inverse()
        let reach = scan([0..n - 1] i => place[rank[z][i]], max)
        for i in 0..n - 1 if reach.prefix(i + 1) == i: good[i] = true
print(1 + good.count(b => b))
