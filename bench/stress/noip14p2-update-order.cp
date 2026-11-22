// NOIP 2014 Joint Weights (proto/ports/noip14p2.cp), with a planted update
// order bug: the top weight is raised before the second-best reads it, so a
// lone neighbour pairs with itself. The edges are read as a `Tree<V>`, which
// stress makes and shrinks as a tree, in place of the port's generator.
// class: mutation

const P = 10007
type Z = Mod<P>
let n: 2..=2e5 = read()
domain V = 1..=n
let edges: Tree<V> = read()
let w: [V]1..=10000 = read()

var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)

fn joint() -> (Int, Z):
    var best = 0
    var total: Z = 0
    for mid in V:
        var top = 0
        var second = 0
        let around = sum(w[u] for u in adj[mid])
        for u in adj[mid]:
            top max= w[u]
            second max= min(top, w[u])
        best max= top * second
        total += Z(around) * Z(around) - sum(Z(w[u] * w[u]) for u in adj[mid])
    (best, total)

@brute(joint)
fn joint_slow() -> (Int, Z):
    var best = 0
    var total: Z = 0
    for mid in V:
        var top = 0
        var second = 0
        let around = sum(w[u] for u in adj[mid])
        for u in adj[mid]:
            second max= min(top, w[u])
            top max= w[u]
        best max= top * second
        total += Z(around) * Z(around) - sum(Z(w[u] * w[u]) for u in adj[mid])
    (best, total)

let (best, total) = joint()
print(best, total)
