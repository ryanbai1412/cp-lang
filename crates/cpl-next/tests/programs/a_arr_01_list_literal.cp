// expect: ok, output "7 1 0 0"
// rule: R04.8.10
// A list where `[V]T` is wanted gives one element per position; a value of the element
// type, `[]` for `[V]List<Int>`, fills every cell instead.
domain V = 0..3
let a: [V]Int = [5, 6, 7]
let g: [V]List<V> = [[V(1)], [V(0)], []]
var e: [V]List<Int> = []
print(a[V(2)], g[V(0)][0], g[V(2)].len, e[V(1)].len)
