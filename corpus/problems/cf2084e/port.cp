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
@!multitest
use std.mod.{Comb, Mod}

type Z = Mod<1_000_000_007>

let n: 1..=5000 = read()
domain Pos = 0..n
domain Val = 0..n
let a: [Pos]?Val = read(none: -1)
let at = a.inverse()

let blanks = prefix([Pos] i => Int(a[i] == none))
let m = blanks.prefix(Pos.last!.r)
let comb = Comb<Z>(n)

var covering: [0..=m]Int = 0
fn tally(ls: Range<Pos>, rs: Range<Pos>, w: Int):
    for l in ls:
        let bl = blanks.prefix(l.l)
        for r in rs if r >= l:
            covering[blanks.prefix(r.r) - bl] += w

tally(Pos.first!..=Pos.last!, Pos.first!..=Pos.last!, 1)
var (lo, hi) = (Pos.last!, Pos.first!)
var need = 0
var answer: Z = 0

for v in Val:
    if let p = at[v]:
        let (lo2, hi2) = (min(lo, p), max(hi, p))
        if let s = lo2.next: tally(s..=lo, hi..=Pos.last!, -1)
        tally(Pos.first!..=lo2, hi..hi2, -1)
        (lo, hi) = (lo2, hi2)
    else:
        need += 1
    let fillings = sum(Z(covering[f]) * comb.arrange(f, need) for f in need..=m)
    answer += fillings * comb.fact(m - need)

print(answer)
