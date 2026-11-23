// expect: ok, output "YES"
// `best_matching` against every subset of edges: the best total, the fewest pairs among best totals
// without `size`, exactly `size` pairs or `None` with it, and `used`/`mate` describing one matching.
use std.flow.{best_matching}
use std.rand.Rng

domain L = 0..3
domain R = 0..3
domain E = 0..6
var rng = Rng(3)
fn r(mut g: Rng, m: Int) -> Int = Int(g.next() % U64(m))
var ok = true
for _ in 0..150:
    let es: [E](l: L, r: R, w: Int) = [E] _ => (
        l: L(r(mut rng, 3)),
        r: R(r(mut rng, 3)),
        w: r(mut rng, 13) - 4,
    )
    // per (min, size): best total and fewest pairs; size -1 means any
    for min in [false, true]:
        for k in -1..5:
            var best: ?(total: Int, pairs: Int) = None
            for mask in 0..1 << E.count:
                var lu: [L]Bool = false
                var ru: [R]Bool = false
                var (fine, tw, n) = (true, 0, 0)
                for e in E:
                    if (mask >> e.ord) & 1 == 0: continue
                    if lu[es[e].l] or ru[es[e].r]: fine = false
                    lu[es[e].l] = true
                    ru[es[e].r] = true
                    tw += es[e].w
                    n += 1
                if not fine or (k >= 0 and n != k): continue
                let better = if let b = best: (
                    if min: tw < b.total else: tw > b.total
                ) or (tw == b.total and n < b.pairs) else: true
                if better: best = Some((total: tw, pairs: n))
            let got = best_matching(es, size: if k >= 0: Some(k) else: None, min:)
            if best == None or got == None:
                ok = ok and best == None and got == None
                continue
            let g = got!
            var lu: [L]Bool = false
            var ru: [R]Bool = false
            var (tw, n) = (0, 0)
            for e in E:
                if not g.used[e]: continue
                ok = ok and not lu[es[e].l] and not ru[es[e].r] and g.mate[es[e].l] == Some(es[e].r)
                lu[es[e].l] = true
                ru[es[e].r] = true
                tw += es[e].w
                n += 1
            for u in L: ok = ok and (g.mate[u] != None) == lu[u]
            ok = ok and tw == g.total and g.total == best!.total and n == best!.pairs
print(if ok: "YES" else: "NO")
