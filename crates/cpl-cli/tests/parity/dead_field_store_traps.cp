domain V = 0..5

struct Paths:
    dist: [V]Int
    back: [V]?(from: V, step: Int)

fn walk(k: Int, steps: List<Int>) -> Paths:
    var dist: [V]Int = 0
    var back: [V]?(from: V, step: Int) = None
    for v in V:
        dist[v] = k * v.to_idx
        back[v] = (from: v, step: steps[v.to_idx])
    Paths(dist, back)

let k = read::<Int>()
print(walk(k, [k, k + 1, k + 2]).dist[V(4)])
