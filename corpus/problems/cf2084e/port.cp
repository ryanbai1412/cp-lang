// Source: Codeforces 2084E (Blossom), https://codeforces.com/contest/2084/problem/E
// limits: 4000 ms, 500 MB
// Problem: a permutation of 0..n-1 with some entries -1 (missing); multitest, sum n <= 5000.
//   Over every way to fill it in, sum the MEX of all non-empty subsegments, mod 1e9 + 7.
// Solution: mex(segment) counts the v with all of 0..=v inside it, so sum over v and over
//   segments the fillings that put 0..=v inside. With `need` of 0..=v unplaced and m blanks, a
//   segment with f blanks covering the span of the placed ones counts P(f, need) (m - need)!.
//   Keep covering[f], the segments with f blanks covering the span; as v grows the span widens
//   and each segment leaves the count once, so O(n^2) per test.
// Bounds: n: 1..=5000
@!multitest
use std.mod.{Comb, Mod}

type Z = Mod<1_000_000_007>

let n: 1..=5000 = read()
domain Pos = 0..n
domain Val = 0..n
let a: [Pos]?Val = read(none: -1)
let at = a.inverse()

let blanks = prefix([Pos] i => Int(a[i] == None))
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
