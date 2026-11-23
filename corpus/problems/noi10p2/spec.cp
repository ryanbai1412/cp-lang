// noi10p2 Super Piano: a chord is a run of lo..=hi consecutive notes; add up the k most
// beautiful. A chord is a pair of prefix cuts. For a start cut and a range of end cuts, the best
// chord ends at the range's highest prefix (a sparse table of (prefix, cut)). `split_best` keeps
// such groups in a heap; taking a group's best splits its range of ends around that cut.
let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], Int(c)))

// Groups and ends are cut counts: `split_best` steps its ranges as `Int`s (it can't take a
// `Range<Note.cuts>`), and `s.shift(len)` (R04.3) isn't implemented yet.
fn best(from: Int, ends: Range<Int>) -> (Int, Int):
    let (p, at) = peak.fold(Note.cuts(ends.start)..=Note.cuts(ends.end - 1))
    (p - pre[Note.cuts(from)], at)

// A generator's `.take(k)` still runs it to the end, so a `for` stops after k.
var (total, left) = (0, k)
for v in split_best(0..=n, range: s => (s + len.start)..min(s + len.end, n + 1), best:):
    total += v
    left -= 1
    if left == 0: break
print(total)
