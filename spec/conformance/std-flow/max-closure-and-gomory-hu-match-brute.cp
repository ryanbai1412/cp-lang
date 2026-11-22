// expect: ok, output "YES"
use std.flow.{max_closure, gomory_hu}
fn mc():
    domain V = 0..6
    for _ in 0..40:
        let before = [V] _ => [V.nth(rand(0, 5)) for _ in 0..rand(0, 2)]
        let p = [V] _ => rand(-6, 6)
        let got = max_closure(before, profit: v => p[v])
        var best = 0
        for m in 0..64:
            var closed = true
            for u in V:
                for v in before[u]:
                    if (m >> v.to_idx) & 1 == 1 and (m >> u.to_idx) & 1 == 0: closed = false
            if closed: best max= sum(p[v] for v in V if (m >> v.to_idx) & 1 == 1)
        assert got.value == best
        assert sum(p[v] for v in V if got.take[v]) == best
        for u in V:
            for v in before[u]: assert not got.take[v] or got.take[u]

fn gh():
    domain V = 0..6
    for _ in 0..30:
        let es = [(u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)), w: rand(0, 7)) for _ in 0..rand(0, 10)]
        let tree = gomory_hu(es)
        assert tree.len == 5
        // the min cut between a and b is the lightest tree edge on their path
        for a in V:
            var light = [V] _ => Int.MAX
            var seen = [V] _ => false
            var stack: List<V> = [a]
            seen[a] = true
            while not stack.is_empty:
                let x = stack.pop()
                for e in tree:
                    for (p, q) in [(e.u, e.v), (e.v, e.u)]:
                        if p == x and not seen[q]:
                            seen[q] = true
                            light[q] = min(light[x], e.w)
                            stack.push(q)
            for b in V:
                if a == b: continue
                var cut = Int.MAX
                for m in 0..64:
                    if (m >> a.to_idx) & 1 == 1 and (m >> b.to_idx) & 1 == 0:
                        cut min= sum(e.w for e in es if ((m >> e.u.to_idx) & 1) != ((m >> e.v.to_idx) & 1))
                assert seen[b] and light[b] == cut

mc()
gh()
print("YES")
