domain V = 0..5

struct Paths:
    dist: [V]Int
    back: [V]?(from: V, step: Int)
    order: List<V>

struct Pair:
    a: Int
    b: Int

fn walk(k: Int, steps: List<Int>) -> Paths:
    var dist: [V]Int = 0
    var back: [V]?(from: V, step: Int) = None
    var order = List<V>()
    for v in V:
        dist[v] = k * v.to_idx
        back[v] = (from: v, step: steps[v.to_idx])
        order.push(v)
    Paths(dist, back, order)

let k = read<Int>()
let p = walk(k, [k, k + 1, k + 2, k + 3, k + 4])
print(p.dist[V(4)])
print(yes(Pair(k, 1) == Pair(k, 2)), yes(Pair(k, 1) < Pair(k, 2)))
