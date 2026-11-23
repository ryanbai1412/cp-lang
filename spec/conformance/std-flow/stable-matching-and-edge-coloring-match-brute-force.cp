// expect: ok, output "YES"
use std.flow.{stable_matching, edge_coloring}

fn random_prefs<A, B>() -> [A]List<B>:
    [A] _ => do:
        var p = [b for b in B if rand(0, 3) > 0]
        for i in 1..p.len:
            let j = rand(0, i)
            let t = p[i]
            p[i] = p[j]
            p[j] = t
        p

fn ranks<A, B>(pref: [A]List<B>) -> [A, B]Int:
    var rk: [A, B]Int = B.count
    for a in A:
        for i in 0..pref[a].len: rk[a, pref[a][i]] = i
    rk

fn is_stable<L, R>(m: [L]?R, rl: [L, R]Int, rr: [R, L]Int) -> Bool:
    var mr: [R]?L = None
    for l in L:
        if let r = m[l]:
            if mr[r] != None or rl[l, r] == R.count or rr[r, l] == L.count: return false
            mr[r] = Some(l)
    for l in L:
        for r in R:
            if rl[l, r] == R.count or rr[r, l] == L.count: continue
            let l_wants = if let x = m[l]: rl[l, r] < rl[l, x] else: true
            let r_wants = if let y = mr[r]: rr[r, l] < rr[r, y] else: true
            if l_wants and r_wants: return false
    true

fn stable_matchings_are_stable_and_proposer_optimal():
    domain L = 0..4
    domain R = 0..3
    for _ in 0..60:
        let pl = random_prefs<L, R>()
        let pr = random_prefs<R, L>()
        let (rl, rr) = (ranks(pl), ranks(pr))
        let m = stable_matching(pl, pr)
        assert is_stable(m, rl, rr)
        let base = R.count + 1
        for code in 0..base ** L.count:
            let other: [L]?R = [L] l => do:
                let d = code / base ** l.to_idx % base
                if d == R.count: None else: Some(R.nth(d))
            if not is_stable(other, rl, rr): continue
            for l in L:
                let mine = if let r = m[l]: rl[l, r] else: R.count
                let theirs = if let r = other[l]: rl[l, r] else: R.count
                assert mine <= theirs

fn edge_colorings_use_max_degree_proper_colours():
    domain L = 0..5
    domain R = 0..4
    domain E = 0..14
    for _ in 0..100:
        let es: [E](l: L, r: R) = [E] _ => (
            l: L.nth(rand(0, L.count - 1)),
            r: R.nth(rand(0, R.count - 1)),
        )
        let (colors, color) = edge_coloring(es)
        var deg = 0
        for l in L: deg = max(deg, [e for e in E if es[e].l == l].len)
        for r in R: deg = max(deg, [e for e in E if es[e].r == r].len)
        assert colors == deg
        for e in E:
            assert 0 <= color[e] and color[e] < colors
            for f in E:
                if e != f and (
                    es[e].l == es[f].l or es[e].r == es[f].r
                ): assert color[e] != color[f]

stable_matchings_are_stable_and_proposer_optimal()
edge_colorings_use_max_degree_proper_colours()
print("YES")
