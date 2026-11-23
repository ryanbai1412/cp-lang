// expect: ok, output "YES"
domain D = 0..9
fn distinct(xs: List<Int>) -> Int:
    var seen = [0..4] _ => false
    for x in xs: seen[x] = true
    seen.count(b => b)
fn piece(a: [D]Int, last: D, len: Int) -> List<Int>:
    var out: List<Int> = []
    for k in 0..len: out.push(a[a.ring.step(last, k - len + 1)])
    out
var ok = true
for _ in 0..20:
    let a = [D] _ => rand(0, 3)
    let mx = a.ring.maximal(state: Counter<Int, Int>(0), ok: c => c.len <= 2)
    let mn = a.ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 6)
    for p in D:
        var best = 0
        for len in 0..=D.count: if distinct(piece(a, p, len)) <= 2: best = len
        if mx[p].len != best or (best > 0 and mx[p].end != p): ok = false
        var short: ?Int = None
        var len = D.count
        while len >= 1:
            if piece(a, p, len).sum() >= 6: short = Some(len)
            len -= 1
        if mn[p].map(w => w.len) != short: ok = false
        if mn[p] != None and mn[p]!.end != p: ok = false
print(yes(ok))
