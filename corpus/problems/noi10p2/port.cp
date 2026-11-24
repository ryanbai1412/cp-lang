// Source: NOI 2010 P2 (Super Piano), https://dmoj.ca/problem/noi10p2
// limits: 1000 ms, 512 MB
// Problem: n <= 5e5 notes of beauty |A_i| <= 1000 and L <= R. Choose k <= 5e5 distinct chords
//   (segments of L to R consecutive notes; enough exist) to maximise their total beauty, the
//   sum of their notes; print it.
// Solution: a chord from cut s is pre[e] - pre[s] for e in a window of cuts; a SparseTable of
//   (prefix sum, cut) gives the best end of any window. A max-heap holds, per start, a window
//   and its best chord; taking the top k times splits its window around the end used into
//   two candidates. O((n + k) log n).
// original: corpus/submissions/5670502.cpp

let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], Int(c)))

// the best chord from cut `from` ending at a cut in `ends`
struct Chord(val: Int, from: Int, ends: Range<Int>, at: Int)
impl Ord for Chord:
    fn lt(a, b) = a.val < b.val

fn chord(from: Int, ends: Range<Int>) -> Chord:
    let (p, at) = peak.fold(Note.cuts(ends.start)..=Note.cuts(ends.end - 1))
    Chord(p - pre[Note.cuts(from)], from, ends, at)

var top = MaxHeap<Chord>()
for s in 0..=n - len.start: top.push(chord(s, (s + len.start)..min(s + len.end, n + 1)))
var total = 0
for _ in 0..k:
    let c = top.pop()!
    total += c.val
    if c.ends.start < c.at: top.push(chord(c.from, c.ends.start..c.at))
    if c.at + 1 < c.ends.end: top.push(chord(c.from, c.at + 1..c.ends.end))
print(total)
