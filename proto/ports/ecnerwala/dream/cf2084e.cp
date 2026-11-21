type Z = Mint(mod: 1_000_000_007)

--- with multitest() ---

domain Pos = read(1..=5000)
let n = Pos.count
let a: [Pos]-1..n = read()

var pos: [0..n]?Pos = none
for i in Pos where a[i] >= 0:
    pos[a[i]] = i

let missing = a.map(x => Int(x == -1)).prefix_sums(at: .cuts)
let m = missing[Pos.Cut.last]
let comb = Z.comb(n)

var freq: [0..=m]Int = 0
for l in Pos:
    for r in l..=Pos.last:
        freq[missing[r.after] - missing[l.before]] += 1

fn add_rect(ls: Range<Pos>, rs: Range<Pos>, delta: Int):
    for l in ls:
        let ml = missing[l.before]
        for r in rs:
            freq[missing[r.after] - ml] += delta

var span: ?(Pos, Pos) = none
var need = 0
var ans = Z(0)

for v in 0..n:
    if let p = pos[v]:
        if let (lo, hi) = span:
            if p < lo:
                add_rect(p.next()!..=lo, hi..=Pos.last, -1)
                span = (p, hi)
            elif p > hi:
                add_rect(Pos.first..=lo, hi..<p, -1)
                span = (lo, p)
        else:
            freq.fill(0)
            add_rect(Pos.first..=p, p..=Pos.last, 1)
            span = (p, p)
    else:
        need += 1
    ans += (need..=m).map(f => Z(freq[f]) * comb.perm(f, need) * comb.fact(m - need)).sum()

print(ans)
