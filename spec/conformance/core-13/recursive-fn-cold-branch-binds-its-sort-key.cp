// expect: ok, output "6"
// rule: 13.3.6
// tool: run --release
// input: "3"
// The cold tail past a recursive call binds a `sort(by:)` key's element
// itself, so the judge C++ of its own frame builds.
let n: 1..=5 = read()
domain V = 1..=n
let tree = Rooted(Graph.undirected([(V(i), V(i + 1)) for i in 1..n]), V.first!)
fn plan(u: V) -> [V]Int:
    let kid_plans = [(c, plan(c)) for c in tree.children(u)]
    let alone: [V]Int = [V] v =>
        var best = 0
        for (c, kid) in kid_plans: best += kid[v]
        best
    var out: [V]Int = 0
    for a in V.sorted(by: a => alone[a]):
        out[a] = alone[a] + u.label
    return out
print(plan(V.first!)[V.first!])
