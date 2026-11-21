// Maintain `freq[f]` = count of subarrays [l..=r] containing every placed value <= v
// and `f` missing (-1) cells. Each placed value extends the required span (removing
// one rectangle of subarrays); each unplaced value increments `need`. Each v adds
// Σ freq[f] · P(f, need) · (m - need)! across completions of the `m` missing cells.
@!multitest
use std.mod.{Comb, Mod}
use std.prefix.prefix1

const P = 1_000_000_007
type Z = Mod<P>

let n: 1..=5000 = read()
domain Pos = 0..n
domain Val = 0..n
let a: [Pos]-1..n = read()

var pos: [Val]?Pos = none
for i in Pos where a[i] >= 0: pos[Val(a[i])] = i

let missing = prefix1([Pos] i => Int(a[i] == -1))
let m = missing.prefix(Pos.last.r)
let comb = Comb<Z>(n)

// Initial triangle (l <= r) cannot be written as one Cartesian rectangle `add_rect`.
var freq: [0..=m]Int = 0
for l in Pos:
    let ml = missing.prefix(l.l)
    for r in l..=Pos.last:
        freq[missing.prefix(r.r) - ml] += 1

fn add_rect(ls: Range<Pos>, rs: Range<Pos>, delta: Int):
    for l in ls:
        let ml = missing.prefix(l.l)
        for r in rs:
            freq[missing.prefix(r.r) - ml] += delta

var span: ?(Pos, Pos) = none
var need = 0
var ans = Z(0)

for v in Val:
    if let p = pos[v]:
        if let (lo, hi) = span:
            if p < lo:
                add_rect(p.next!..=lo, hi..=Pos.last, -1)
                span = (p, hi)
            elif p > hi:
                add_rect(Pos.first..=lo, hi..p, -1)
                span = (lo, p)
        else:
            freq.fill(0)
            add_rect(Pos.first..=p, p..=Pos.last, 1)
            span = (p, p)
    else:
        need += 1
    let ways_rest = comb.fact(m - need)
    ans += sum(Z(freq[f]) * comb.arrange(f, need) * ways_rest for f in need..=m)

print(ans)
