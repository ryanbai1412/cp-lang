// mex(segment) counts the v ≥ 1 with all of 0..v inside it, so the answer sums,
// over v and segments, the fillings that put every value below v inside. With f
// blanks in the segment, the `need` unplaced values below v go to those blanks in
// P(f, need) ways and the other blanks fill in (m − need)! ways, provided every
// placed value below v is inside: the segment covers their span lo..=hi. Count
// segments by blanks; as v grows the span only widens, and each segment drops
// out once, when the span first leaves it. O(n²) per test.
type Z = Mod<1e9 + 7>

--- multitest ---
domain Pos = read(1..=5000)
let a: [Pos]?(0..Pos.count) = read(none: -1)
let at = a.inverse()                                   // position of each placed value
let blanks = a.map(x => x == none).prefix_sums(at: .cuts)
fn blank(l: Pos, r: Pos) -> Int = blanks[r.after] - blanks[l.before]
let m = blanks.last
let comb = Z.comb(Pos.count)

var covering: [0..=m]Int = 0                           // segments covering the span, by blanks
for l in Pos:
    for r in l..=Pos.last: covering[blank(l, r)] += 1
var (lo, hi) = (Pos.last, Pos.first)                   // empty span: every segment covers it
var need = 0
var answer: Z = 0
for v in 0..Pos.count:                                 // term for mex > v
    if let p = at[v]:
        let (lo2, hi2) = (min(lo, p), max(hi, p))
        for l in (lo2..=lo).skip(1):
            for r in max(l, hi)..=Pos.last: covering[blank(l, r)] -= 1
        for l in Pos.first..=lo2:
            for r in max(l, hi)..hi2: covering[blank(l, r)] -= 1
        (lo, hi) = (lo2, hi2)
    else:
        need += 1
    answer += sum(Z(covering[f]) * comb.arrange(f, need) for f in need..=m) * comb.fact(m - need)
print(answer)
