// A small tuple unpacks into Int registers on the VM, not through the
// tree-walker: from a boxed tuple parameter, from a local array, and from an
// array that is a struct's field, by one index or two.
domain V = 0..3
struct Pairs(pub at: [V](Int, Int), pub grid: [V, V](Int, Int))

fn spread(t: (Int, Int)) -> Int:
    let (a, b) = t
    return a * 7 + b

let p = Pairs([V] v => (Int(v), Int(v) * 10), [V, V] (u, v) => (Int(u), Int(v)))
let local = [V] v => (Int(v) + 1, Int(v) + 2)
var total = 0
for v in V:
    let (a, b) = p.at[v]
    let (c, d) = p.grid[v, V(1)]
    let (e, f) = local[v]
    total += a + b + c * 100 + d * 1000 + e * e + f + spread(p.at[v])
print(total)
