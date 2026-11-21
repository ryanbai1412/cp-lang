// Maintain `freq[f]` = count of subarrays [l..=r] containing every placed value <= v
// and `f` missing (-1) cells. As v grows from 0 to n-1, either v is placed (so valid
// subarrays must cover its position, removing one rectangle of (l, r) pairs as the
// required span extends) or v is unplaced (incrementing `need`). Each v contributes
// Σ freq[f] · P(f, need) · (m - need)! across all completions of the missing cells.
@!multitest
type Z = Mod<1_000_000_007>

domain Pos = read(1..=5000)
domain Val = 0..Pos.count
let a: [Pos]-1..Pos.count = read()

var pos: [Val]?Pos = none
for i in Pos where a[i] >= 0: pos[Val(a[i])] = i

let missing = a.map(x => Int(x == -1)).prefix_sums(at: .cuts)
let m = missing[Pos.Cut.last]
let comb = Z.comb(Pos.count)

var freq: [0..=m]Int = 0
for l in Pos:
    for r in l..=Pos.last:
        freq[missing[r.after] - missing[l.before]] += 1

struct Span:
    bounds: ?(Pos, Pos)

    // Extend the covered span to include `p`; returns the rectangle of subarrays
    // that no longer contain all placed values (or `none` if already covered).
    fn extend(mut self, p: Pos) -> ?(Range<Pos>, Range<Pos>):
        if let (lo, hi) = bounds:
            if p < lo:
                bounds = (p, hi)
                (p.next!..=lo, hi..=Pos.last)
            elif p > hi:
                bounds = (lo, p)
                (Pos.first..=lo, hi..p)
            else:
                none
        else:
            bounds = (p, p)
            none

var span = Span(none)
var need = 0
var ans: Z = 0
for v in Val:
    if let p = pos[v]:
        if span.bounds == none:
            freq.fill(0)
            for l in Pos.first..=p:
                for r in p..=Pos.last:
                    freq[missing[r.after] - missing[l.before]] += 1
        if let (ls, rs) = span.extend(p):
            for l in ls:
                for r in rs:
                    freq[missing[r.after] - missing[l.before]] -= 1
    else:
        need += 1
    ans += sum(Z(freq[f]) * comb.arrange(f, need) * comb.fact(m - need) for f in need..=m)
print(ans)
