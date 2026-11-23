// CF 2027D2 The Endspeaker (hard): dp over (level, removed prefix) of (min cost, ways).
// At level t a prefix removal from j' to j costs m - t and needs sum(a[j'..j]) <= b[t];
// the valid j' form a window [lo, j), so a segment tree folds (min, ways) over it.
type Z = Mod<1_000_000_007>

struct Best:
    cost: Int
    ways: Z

fn join(x: Best, y: Best) -> Best:
    if x.cost < y.cost: x elif y.cost < x.cost: y else: Best(cost: x.cost, ways: x.ways + y.ways)

let none = Best(cost: 1 << 62, ways: Z(0))

@!multitest
let (n, m): (1..=3e5, 1..=3e5) = read()
let a = read<[n]1..=1e9>()
let b = read<[m]1..=1e9>()
let pre = a.scan(+)
domain J = 0..=n
var prev: [J]Best = none
prev[J.first!] = Best(cost: 0, ways: Z(1))
var ans = none
for t in 0..m:
    var st = SegTree<J, Best>(Monoid(none, join), fill: none)
    var cur: [J]Best = none
    var lo = 0
    for j in J:
        let jj = Int(j)
        var v = if jj < n: prev[j] else: none
        while pre[jj] - pre[lo] > b[t]:
            lo += 1
        if lo < jj:
            let w = st.fold(J(lo)..j)
            if w.cost < none.cost:
                v = join(v, Best(cost: w.cost + (m - 1 - t), ways: w.ways))
        cur[j] = v
        st.set(j, v)
    ans = join(ans, cur[J.last!])
    prev = cur
if ans.cost == none.cost:
    print(-1)
else:
    print(ans.cost, ans.ways)
