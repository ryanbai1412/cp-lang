// NOI 2010 Super Piano: a chord is len_min..=len_max consecutive notes and
// its beauty is the sum of theirs. Add up the k most beautiful chords.
// original: corpus/submissions/5670502.cpp
let n: 1..=5e5 = read()
let k: 1..=5e5 = read()
let len_min: 1..=n = read()
let len_max: len_min..=n = read()
domain Note = 1..=n
domain Cut = 0..=n                          // cut c is after note c
let chords = sum(min(n, s + len_max) - (s + len_min) + 1 for s in 0..=n - len_min)
let beauty: [Note]-1000..=1000 = read() where k <= chords

var upto: [Cut]Int = 0                      // total beauty before each cut
for c in Cut:
    if let b = c - 1: upto[c] = upto[b] + beauty[Note(c.label)]

// A chord is a pair of cuts. For a start cut and a range of end cuts, the
// best chord ends at the range's highest `upto`. The best chord overall is the
// top of a heap of such ranges, and taking it splits its range in two.
let highest = SegTree<Cut, (Int, Cut)>((-1e18, Cut.first), (a, b) => max(a, b))
for c in Cut: highest.set(c, (upto[c], c))

type Offer = (loss: Int, start: Cut, lo: Cut, hi: Cut, end: Cut)    // loss = -beauty: only a MinHeap exists
var offers = MinHeap<Offer>()

fn offer(start: Cut, lo: Cut, hi: Cut):
    let (top, end) = highest.query(lo, hi)
    offers.push((upto[start] - top, start, lo, hi, end))

for start in Cut:
    if start.label + len_min <= n:
        offer(start, Cut(start.label + len_min), Cut(min(n, start.label + len_max)))

var total = 0
for _ in 0..k:
    if let (loss, start, lo, hi, end) = offers.pop():
        total -= loss
        if lo < end: offer(start, lo, Cut(end.label - 1))
        if end < hi: offer(start, Cut(end.label + 1), hi)
print(total)
