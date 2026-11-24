// Tuned: one [cut, base, base] count grid filled in a single pass, in place
// of nine Prefix objects, and each query reads the grid directly instead of
// building a [Base, Base] table and a list over Pair<Base>.
let (n, q): (1..=100_000, 1..=100_000) = read()
domain Pos = 0..n
domain Base = 0..3
fn base(c: Char) -> Base = Base(if c == 'A': 0 elif c == 'C': 1 else: 2)
let (a, b): (Str, Str) = read()
assert a.len == n and b.len == n

var count: [Pos.cuts, Base, Base]Int = 0
for i in Pos:
    for s in Base:
        for t in Base: count[i.r, s, t] = count[i.l, s, t]
    count[i.r, base(a[Int(i)]), base(b[Int(i)])] += 1

fn have(x: Pos, y: Pos, s: Base, t: Base) -> Int = count[y.r, s, t] - count[x.l, s, t]

for _ in 0..q:
    let (x, y): (Pos, Pos) = read()
    assert x <= y
    if any(sum(have(x, y, s, t) for t in Base) != sum(have(x, y, t, s) for t in Base) for s in Base):
        print(-1)
    else:
        var (swaps, wrong) = (0, 0)
        for s in Base:
            for t in Base:
                if s < t: swaps += min(have(x, y, s, t), have(x, y, t, s))
                if s != t: wrong += have(x, y, s, t)
        print(swaps + (wrong - 2 * swaps) / 3 * 2)
