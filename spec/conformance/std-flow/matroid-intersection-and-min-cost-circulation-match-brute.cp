// expect: ok, output "YES"
use std.flow.{GraphicMatroid, ColorMatroid, matroid_intersection, min_cost_circulation}

fn matroid_intersections_match_brute_force():
    domain V = 0..4
    domain E = 0..7
    domain C = 0..3
    for _ in 0..120:
        let es: [E](u: V, v: V) = [E] _ => (u: V.nth(rand(0, 3)), v: V.nth(rand(0, 3)))
        let col: [E]C = [E] _ => C.nth(rand(0, 2))
        let lim: [C]Int = [C] _ => rand(0, 2)
        let wt: [E]Int = [E] _ => rand(-3, 5)
        var best_n = -1
        var best_w = 0
        for mask in 0..1 << E.count:
            let s: [E]Bool = [E] e => (mask >> e.ord) & 1 == 1
            var d = Dsu<V>()
            var used: [C]Int = 0
            var ok = true
            var n = 0
            var tw = 0
            for e in E:
                if not s[e]: continue
                if not d.unite(es[e].u, es[e].v): ok = false
                used[col[e]] += 1
                if used[col[e]] > lim[col[e]]: ok = false
                n += 1
                tw += wt[e]
            if not ok: continue
            if n > best_n or (n == best_n and tw > best_w):
                best_n = n
                best_w = tw
        for weighted in [false, true]:
            let got = matroid_intersection(
                GraphicMatroid(es),
                ColorMatroid(col, lim),
                weight: if weighted: (e: E) => wt[e] else: _ => 0,
            )
            var d = Dsu<V>()
            var used: [C]Int = 0
            var n = 0
            var tw = 0
            for e in E:
                if not got[e]: continue
                assert d.unite(es[e].u, es[e].v)
                used[col[e]] += 1
                assert used[col[e]] <= lim[col[e]]
                n += 1
                tw += wt[e]
            assert n == best_n
            if weighted: assert tw == best_w

fn min_cost_circulations_match_brute_force():
    domain V = 0..4
    domain E = 0..5
    for _ in 0..150:
        let es: [E](u: V, v: V, lo: Int, hi: Int, cost: Int) = [E] _ =>
            let lo = rand(0, 1)
            (
                u: V.nth(rand(0, 3)),
                v: V.nth(rand(0, 3)),
                lo:,
                hi: lo + rand(0, 2),
                cost: rand(-4, 4),
            )
        var best: ?Int = None
        var f: [E]Int = 0
        for code in 0..3 ** E.count:
            var ok = true
            var net: [V]Int = 0
            for e in E:
                f[e] = es[e].lo + code / 3 ** e.ord % 3
                if f[e] > es[e].hi: ok = false
                net[es[e].u] -= f[e]
                net[es[e].v] += f[e]
            if not ok or [v for v in V if net[v] != 0].len > 0: continue
            var c = 0
            for e in E: c += f[e] * es[e].cost
            if best == None or c < best!: best = Some(c)
        let got = min_cost_circulation(es)
        assert (got?.cost) == best
        if let r = got:
            var net: [V]Int = 0
            for e in E:
                assert es[e].lo <= r.flow[e] and r.flow[e] <= es[e].hi
                net[es[e].u] -= r.flow[e]
                net[es[e].v] += r.flow[e]
            for v in V: assert net[v] == 0

matroid_intersections_match_brute_force()
min_cost_circulations_match_brute_force()
print("YES")
