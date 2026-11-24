// NOI 2010 Super Piano: the k best chords (L..=R consecutive notes) by total beauty. A chord from
// cut s ending at cut e is pre[e] - pre[s]. Keep, per start, the best end of a window of ends in
// a heap; taking it splits the window around that end.
let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], Int(c)))

// (value, start, ends lo..hi, best end) for the chords from s ending in lo..hi
fn chord(s: Int, lo: Int, hi: Int) -> (Int, Int, Int, Int, Int):
    let (p, at) = peak.fold(Note.cuts(lo)..=Note.cuts(hi - 1))
    (p - pre[Note.cuts(s)], s, lo, hi, at)

var top = MaxHeap<(Int, Int, Int, Int, Int)>()
for s in 0..=n - len.start: top.push(chord(s, s + len.start, min(s + len.end, n + 1)))
var total = 0
repeat k:
    let (v, s, lo, hi, at) = top.pop()!
    total += v
    if lo < at: top.push(chord(s, lo, at))
    if at + 1 < hi: top.push(chord(s, at + 1, hi))
print(total)
