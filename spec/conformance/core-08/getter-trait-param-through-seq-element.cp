// expect: ok, output "2 1\n0"
// rule: 08.1.7
// A getter trait on a Seq element binds its parameter from the record field, through `filter` and extra fields.
trait EdgeEnds<V>:
    @get fn u(self) -> V
    @get fn v(self) -> V
fn adj2<S: Seq<E>, E: EdgeEnds<V>, V>(edges: S) -> [V]List<V>:
    var adj: [V]List<V> = []
    for e in edges: adj[e.u].push(e.v)
    adj
domain V = 0..4
let es = [(u: V(0), v: V(2), w: 5), (u: V(0), v: V(1), w: 7), (u: V(3), v: V(0), w: 1)]
print(adj2(es)[V(0)])
print(adj2(es.filter(e => e.w > 2))[V(3)].len)
