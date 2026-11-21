// Codeforces 2084 E: Blossom.
//
// mex(segment) = the number of v >= 0 with all of 0..=v inside it, so the
// answer sums, over v and over segments, the fillings that put every value
// in 0..=v inside the segment. Let `need` be how many of 0..=v are unplaced
// and m the number of blanks. A segment with f blanks counts
// P(f, need) * (m - need)! fillings (the missing values go to its blanks,
// the rest anywhere), provided it covers the span lo..=hi of the placed
// values in 0..=v.
//
// So keep `covering[f]`, the segments with f blanks covering the span. As v
// grows the span only widens; when a value at p widens it to lo2..=hi2, the
// segments that stop covering it start in (lo2, lo] or end in [hi, hi2).
// Each segment leaves once, so the whole walk is O(n^2) per test.
type Z = Mod<1e9 + 7>

@!multitest
domain Pos = read(1..=5000)
let a: [Pos]?(0..Pos.count) = read(none: -1)
let at = a.inverse()
let blanks = a.map(x => Int(x == none)).scan(+)
fn blank(l: Pos, r: Pos) -> Int = blanks[r.after] - blanks[l.before]
let m = blanks.last
let comb = Z.comb(Pos.count)

var covering: [0..=m]Int = 0
fn tally(ls: Range<Pos>, rs: Range<Pos>, w: Int):
    for l in ls:
        for r in rs if r >= l:
            covering[blank(l, r)] += w

tally(Pos, Pos, 1)
var (lo, hi) = (Pos.last, Pos.first)                   // no value placed yet: every segment covers
var need = 0
var answer: Z = 0
for v in 0..Pos.count:
    if let p = at[v]:
        let (lo2, hi2) = (min(lo, p), max(hi, p))
        tally((lo2..=lo).drop(1), hi..=Pos.last, -1)
        tally(Pos.first..=lo2, hi..hi2, -1)
        (lo, hi) = (lo2, hi2)
    else:
        need += 1
    let fillings = sum(Z(covering[f]) * comb.arrange(f, need) for f in need..=m)
    answer += fillings * comb.fact(m - need)
print(answer)
