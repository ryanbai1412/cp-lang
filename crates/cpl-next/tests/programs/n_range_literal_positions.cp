// expect: ok, output "2 b 1\n2 3\n3"
// rule: R04.4.1 R04b.1.2
// A literal range where a range of a declared domain's positions is wanted takes its ends as
// positions, as a literal index does: `t[1..3]` on `[V]T` and on `[0..4]T`. The piece is read by
// position too, wherever its type was inferred.
domain V = 0..4
let t: [V]Char = ['a', 'b', 'b', 'c']
let w = t[1..3]
print(w.len, w[V(1)], w.keys.first!)
let u: [0..4]Int = [1, 2, 3, 4]
print(u[1..3].len, u[1..3].sum() - 2)
let r: Range<V> = 1..=3
print(r.len)
