// expect: ok, output "YES\nYES\nYES\nYES\nYES\nYES YES YES"
use std.geometry.{Pt, convex_hull, farthest_pair, hull_line_intersect, manhattan_mst}
domain D3 = 0..3
domain D2 = 0..2
domain K = 0..8
let p3 = [Pt(0, 0), Pt(1, 1), Pt(3, 0)]
let fp = farthest_pair([D3] i => p3[i.to_idx])
print(yes(fp.a == D3(0) and fp.b == D3(2) and fp.dist2 == 9))
print(yes(hull_line_intersect([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(-1, 1), Pt(3, 1)) == Some((1, 3))))
print(yes(hull_line_intersect([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(-1, 5), Pt(3, 5)) == None))
let p5 = [Pt(0, 0), Pt(1, 1), Pt(5, 5)]
print(yes(manhattan_mst([D3] i => p5[i.to_idx]).any(e => e.u == D3(0) and e.v == D3(1) and e.w == 2)))
let p2 = [Pt(0, 0), Pt(2, 0)]
let m2 = manhattan_mst([D2] i => p2[i.to_idx])
print(yes(m2.len == 1 and m2[0].u == D2(0) and m2[0].v == D2(1) and m2[0].w == 2))
var seed = 7
fn rnd(mut seed: Int, n: Int) -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed % n
var okf = true
var okm = true
var okl = true
for _ in 0..80:
    let n = 8
    let ps = [Pt(rnd(mut seed, 9) - 4, rnd(mut seed, 9) - 4) for _ in 0..n]
    let kp = [K] i => ps[i.to_idx]
    var bd = -1
    var ba = 0
    var bb = 0
    for i in 0..n:
        for j in i + 1..n:
            if ps[i].dist2(ps[j]) > bd:
                bd = ps[i].dist2(ps[j])
                ba = i
                bb = j
    let f = farthest_pair(kp)
    if f.a.to_idx != ba or f.b.to_idx != bb or f.dist2 != bd: okf = false
    var inn = List.filled(n, false)
    var dist = List.filled(n, Int.MAX)
    dist[0] = 0
    var total = 0
    for _ in 0..n:
        var u = -1
        for v in 0..n:
            if not inn[v] and (u < 0 or dist[v] < dist[u]): u = v
        inn[u] = true
        total += dist[u]
        for v in 0..n:
            if ps[u].manhattan(ps[v]) < dist[v]: dist[v] = ps[u].manhattan(ps[v])
    var comp = [i for i in 0..n]
    fn root(comp: List<Int>, x: Int) -> Int:
        var r = x
        while comp[r] != r: r = comp[r]
        r
    var got = 0
    for e in manhattan_mst(kp):
        let a = root(comp, e.u.to_idx)
        let b = root(comp, e.v.to_idx)
        if a != b:
            comp[a] = b
            got += e.w
    if got != total: okm = false
    let h = convex_hull(ps)
    if h.len >= 3:
        let a = Pt(rnd(mut seed, 11) - 5, rnd(mut seed, 11) - 5)
        let b = Pt(rnd(mut seed, 11) - 5, rnd(mut seed, 11) - 5)
        if a != b:
            let d = b - a
            let m = h.len
            var pos = false
            var neg = false
            var cuts: List<Int> = []
            for k in 0..m:
                let s1 = d.cross(h[k] - a)
                let s2 = d.cross(h[(k + 1) % m] - a)
                if s1 > 0: pos = true
                if s1 < 0: neg = true
                if (s1 > 0 and s2 <= 0) or (s1 < 0 and s2 >= 0): cuts.push(k)
            let r = hull_line_intersect(h, a, b)
            if pos and neg:
                if r == None or cuts != [r!.0, r!.1]: okl = false
            else:
                if r != None: okl = false
print(yes(okf), yes(okm), yes(okl))
