// noi10p2 Super Piano: a chord is a run of lo..=hi consecutive notes; add up the k most
// beautiful. A chord is a pair of prefix cuts. For a start cut and a range of end cuts, the best
// chord ends at the range's highest prefix (a sparse table of (prefix, cut)). A heap holds such
// groups; taking a group's best splits its range of ends around that cut.
let (n, k) = read<(1..=500_000, 1..=500_000)>()
let (lo, hi) = read<(1..=n, 1..=n)>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let tops = SparseTable(max, [Note.cuts] c => (pre[c], c))
type Group = (beauty: Int, from: Note.cuts, first: Note.cuts, last: Note.cuts, at: Note.cuts)
fn group(from: Note.cuts, first: Note.cuts, last: Note.cuts) -> Group:
    let (p, at) = tops.fold(first..=last)
    (beauty: p - pre[from], from: from, first: first, last: last, at: at)
var heap = MaxHeap<Group>(by: g => g.beauty)
for from in Note.cuts:
    if let first = from.next(lo):
        heap.push(group(from, first, from.next(hi) ?? Note.cuts.last!))
var total = 0
for _ in 0..k:
    let g = heap.pop()!
    total += g.beauty
    if g.first < g.at:
        heap.push(group(g.from, g.first, g.at.prev!))
    if g.at < g.last:
        heap.push(group(g.from, g.at.next!, g.last))
print(total)
