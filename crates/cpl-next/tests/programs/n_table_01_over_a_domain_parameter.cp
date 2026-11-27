// expect: ok, output "2 1\n0 0 0\n3"
// `[D] k => e` over a domain type parameter walks `D`'s positions in each instance, as over a
// declared domain (R02.8.2, R04.4.30).
fn degs<L>(adj: [L]List<Int>) -> [L]Int = [L] l => adj[l].len
fn zeros<D: Domain>() -> [D]Int = [D] _ => 0
fn ords<D: Domain>() -> [D]Int = [D] k => k.ord
domain V = 0..3
let adj: [0..2]List<Int> = [[0, 1], [0]]
print(degs(adj))
print(zeros<V>())
print(ords<V>().sum())
