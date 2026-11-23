// noi10p2 Super Piano: a chord is a run of lo..=hi consecutive notes; add up the k most
// beautiful. A chord is a pair of prefix cuts. For a start cut and a range of end cuts, the best
// chord ends at the range's highest prefix (a sparse table of (prefix, cut)). A heap holds such
// groups; taking a group's best splits its range of ends around that cut.
let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], c))

fn best(from: Note.cuts, ends: Range<Note.cuts>) -> (Int, Note.cuts):
    let (p, at) = peak.fold(ends)
    (p - pre[from], at)

print(split_best(Note.cuts, range: s => s.shift(len), best:).take(k).sum())
