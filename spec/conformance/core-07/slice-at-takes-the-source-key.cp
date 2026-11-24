// expect: ok, output "20 30\n3 4"
// rule: 07.4.17
// A slice's `at` is its `Indexed` impl's: it takes the source's key, a `V` on a `[V]T` slice, and reads without needing a `var` source.
domain V = 1..=5
let a = [V] v => v.ord * 10
let s = a[V(2)..=V(4)]
print(s.at(V(3)), s.at(V(4)))
let xs = [1, 2, 3, 4, 5]
print(xs[1..4].at(2), xs[1..4].at(3))
