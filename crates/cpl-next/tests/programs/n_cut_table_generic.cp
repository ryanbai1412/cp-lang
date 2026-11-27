// expect: ok, output "3 3 3 1\n0 1"
// rule: R04.2.17
// A `[V.cuts]T` table takes cuts as keys, also in a body bound only `V: Domain`: `V.cuts(k)` is
// the cut with `k` positions before it and a walk of `V.cuts` steps with `c.prev`, as
// std.graph's offsets do.
fn tally<V: Domain>(xs: List<V>) -> [V.cuts]Int:
    var s: [V.cuts]Int = 0
    for v in xs: s[V.cuts(v.ord)] += 1
    for c in V.cuts:
        if let p = c.prev: s[c] += s[p]
    s
domain V = 0..4
let s = tally([V(1), V(3), V(3)])
print(s[V.cuts.last!], s[V(3).l], s[V(2).r], s[V.cuts(1)])
print(s[V.cuts(0)], s[V(0).r])
