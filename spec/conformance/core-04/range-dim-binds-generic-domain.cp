// expect: ok, output "2 1"
// rule: 04.4.30
// An integer range used as an array dimension is an anonymous domain, so it binds a generic
// domain parameter like a declared one.
fn degs<L>(adj: [L]List<Int>) -> [L]Int = [L] l => adj[l].len
let adj: [0..2]List<Int> = [[0, 1], [0]]
print(degs(adj))
