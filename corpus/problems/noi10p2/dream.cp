// noi10p2 Super Piano: a chord is lo..=hi consecutive notes; add up the k most beautiful. A chord
// is a pair of prefix cuts. For a start i and a range of end cuts, the best chord ends at the
// range's highest prefix (a sparse table of (prefix, cut)). A heap holds such groups; taking a
// group's best splits its range around that end.
let (n, k) = read<(1..=500_000, 1..=500_000)>()
let (lo, hi) = read<(1..=n, 1..=n)>()
let pre = read<[n]-1_000..=1_000>().scan(+)
let tops = SparseTable(max, [0..=n] j => (pre[j], j))
type Group = (beauty: Int, i: Int, lo: Int, hi: Int, at: Int)
fn group(i: Int, l: Int, r: Int) -> Group:
    let (p, j) = tops.fold(l..=r)
    (beauty: p - pre[i], i: i, lo: l, hi: r, at: j)
var heap = MaxHeap<Group>(by: g => g.beauty)
for i in 0..=n - lo:
    heap.push(group(i, i + lo, min(i + hi, n)))
var total = 0
for _ in 0..k:
    let g = heap.pop()!
    total += g.beauty
    if g.lo < g.at:
        heap.push(group(g.i, g.lo, g.at - 1))
    if g.at < g.hi:
        heap.push(group(g.i, g.at + 1, g.hi))
print(total)
