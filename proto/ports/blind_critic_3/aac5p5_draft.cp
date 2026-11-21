// original: 4240171 (aac5p5) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
type H = (y: Int, x1: Int, x2: Int)
type Vt = (x: Int, y1: Int, y2: Int)
let (n, m): (Int, Int) = read()
let hori: [0..n]H = read()
let vert: [0..m]Vt = read()
var xs: List<Int> = []
for h in hori:
    xs.push(h.x1)
    xs.push(h.x2)
for v in vert: xs.push(v.x)
xs.sort()
dedup(mut xs)
let K = xs.len
fn cx(x: Int) -> Int: return xs.first_ge(x)!
var by_y: Map<Int, List<(Int, Int)>> = Map<Int, List<(Int, Int)>>()
for h in hori: (by_y[h.y] ??= []).push((cx(h.x1), cx(h.x2)))
fn c2(x: Int) -> Int: return x * (x - 1) / 2
var ans = 0
for (y1, s1) in by_y.items:
    var cnt: [0..K]Int = 0
    var b1 = Fenwick<0..K, Int>()
    var b2 = Fenwick<0..K, Int>()
    fn upd(i: Int, d: Int):
        b1.add(i, d)
        let old = c2(cnt[i])
        cnt[i] += d
        b2.add(i, c2(cnt[i]) - old)
    var dels: List<(Int, Int)> = []
    for v in vert:
        if v.y1 <= y1 and y1 < v.y2:
            upd(cx(v.x), 1)
            dels.push((v.y2, cx(v.x)))
    dels.sort()
    var j = 0
    for (y2, s2) in by_y.items:
        if y2 <= y1: continue
        while j < dels.len:
            let (dy, dx) = dels[j]
            if dy >= y2: break
            upd(dx, -1)
            j += 1
        for (l1, r1) in s1:
            for (l2, r2) in s2:
                let l = max(l1, l2)
                let r = min(r1, r2)
                if l <= r: ans += c2(b1.sum(l..=r)) - b2.sum(l..=r)
print(ans)
