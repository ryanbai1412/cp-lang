// expect: ok, output "3 1\n0 3\n4 1 0 2\n0 1 2 3"
use std.strings.{lyndon_factorization, min_rotation, manacher, z_function, prefix_function}
domain P = 0..4
let a: [P]Char = [P] p => "baba"[p.ord]
let b: [P]Char = [P] p => "aaaa"[p.ord]
print(lyndon_factorization(a).len, min_rotation(a)!.ord)
print(manacher(a).longest())
print(z_function(a)[P(0)], z_function(b)[P(3)], z_function(a)[P(1)], z_function(a)[P(2)])
print(prefix_function(b)[P(0)], prefix_function(b)[P(1)], prefix_function(b)[P(2)], prefix_function(b)[P(3)])
