// expect: ok, output "0 0 2 1"
// rule: 02.10.1
// `[]` takes its type from where it goes (one cell when it fills an array), `rand(lo, hi)` is an `Int`, and `push` gives `()`.
domain V = 0..2
var adj: [V]List<Int> = []
let e: List<Int> = []
var a = [1]
a.push(2)
let r = rand(1, 1)
print(adj[V(0)].len, e.len, a.len, r)
