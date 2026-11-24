// expect: E0203
// rule: std.graph.Next
// `Next` gives `out` to a table of walks; a table of `Int`s is not one, so it has no `out`.
domain V = 0..3
let a: [V]Int = [V] v => 0
print(a.out(V(1))) // <- here
