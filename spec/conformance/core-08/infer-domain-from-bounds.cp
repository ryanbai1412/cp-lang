// expect: ok, output "2 1"
// rule: 08.1.1
// Generic domain `V` is inferred from row bounds on static methods and from
// trait-typed parameters over domain-indexed tables without defaulting to Int.
trait Adj<V>:
    fn out(self, u: V) -> List<V>

struct Net<V>:
    n: Int

    pub fn build<E, R: (u: V, v: V)>(edges: [E]R) -> Net<V>:
        Net(edges.len)

    pub fn count(self, v: V) -> Int = n + v.ord

fn deg_of<V>(g: Adj<V>) -> [V]Int:
    var out: [V]Int = [V] _ => 0
    for v in V:
        out[v] = g[v].len
    out

domain V = 0..3
domain E = 0..2
let edges: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 5), (u: V(1), v: V(2), w: 3)]
let net = Net.build(edges)
let adj: [V]List<V> = [[V(1)], [V(2)], []]
let ds = deg_of(adj)
print(net.count(V(0)), ds[V(1)])
