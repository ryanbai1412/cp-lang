// cp: 0.1 af7fc88+dirty
// BOI 2018 Paths: count simple paths of at least two vertices whose colours
// are all distinct (at most 5 colours, so a path has at most 5 vertices).
let n: 1..=3e5 = read()
let m: 0..=3e5 = read()
let k: 1..=5 = read()
domain V = 1..=n
domain Colour = 1..=k
type Used = Subset<Colour>
let colour: [V]Colour = read()
let edges: [m](u: V, v: V) = read()

var adj: [V]List<V> = []
for (u, v) in edges:
    adj[u].push(v)
    adj[v].push(u)

// ways[s, v]: paths ending at v whose colours are exactly the set s.
// A subset comes after all its own subsets, so one pass in order fills it.
var ways: [Used, V]Int = 0
for s in Used:
    for v in V:
        if colour[v] in s:
            let rest = s.without(colour[v])
            ways[s, v] = if rest.is_empty: 1 else: sum(ways[rest, u] for u in adj[v])
print(sum(ways[s, v] for s in Used for v in V if s.len >= 2))
