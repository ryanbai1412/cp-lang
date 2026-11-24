// Tuned: port.cp with the blank counts as a plain prefix array in 2 bytes (no Prefix
// table to branch on per read) and covering[f] as a segment count, at most n(n+1)/2,
// in 4 bytes like orig.cpp's `int` freq, so the O(n^2) tally loop is narrow loads and stores.
@!multitest
use std.mod.{Comb, Mod}
use std.prefix.{prefix_fold}

type Z = Mod<1_000_000_007>

let n: 1..=5000 = read()
domain Pos = 0..n
domain Val = 0..n
let a: [Pos]?Val = read(none: -1)
let at = a.inverse()

let blanks: [Pos.cuts]0..=5000 = prefix_fold(+, [Pos] i => Int(a[i] == None))
let m = blanks[Pos.last!.r]
let comb = Comb<Z>(n)

var covering: [0..=m]0..=12_502_500 = 0
fn tally(ls: Range<Pos>, rs: Range<Pos>, w: Int):
    for l in ls:
        let bl = blanks[l.l]
        for r in rs if r >= l:
            covering[blanks[r.r] - bl] += w

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
