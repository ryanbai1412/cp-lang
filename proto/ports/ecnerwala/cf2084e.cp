@!multitest
use std.mod.{Comb, Mod}
use std.prefix.prefix1

type Z = Mod<1_000_000_007>

let n: 1..=5000 = read()
domain Pos = 0..n
domain Val = 0..n
let a: [Pos]Int = read()

var pos: [Val]?Pos = none
for i in Pos where a[i] >= 0:
    pos[Val(a[i])] = i

let missing = prefix1([Pos] i => if a[i] == -1: 1 else: 0)
let m = missing.prefix(Pos.last.r)
let comb = Comb<1_000_000_007>(n)

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
                add_rect(Pos(p.to_idx + 1)..=lo, hi..=Pos.last, -1)
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
