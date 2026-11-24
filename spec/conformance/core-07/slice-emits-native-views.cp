// expect: ok
// rule: 07.4.17
// tool: build -o -
// has: "cpl::ListView<ll> w;"
// has: "cpl::ListView<ll> d;"
// has: "cpl::StrView t;"
// has: "w = cpl::list_slice(xs, 1, 4);"
// has: "t = cpl::str_slice(s, 2, 5);"
// lacks: "to_list(xs"
// A slice of a List, a [D]T and a Str is a pointer view in C++: no copy of the source.
domain V = 0..5
let xs = [1, 2, 3, 4, 5]
let a: [V]Int = [10, 20, 30, 40, 50]
let s = "abcdef"
let w = xs[1..4]
let d = a[V(1)..V(3)]
let t = s[2..5]
print(w.len, d.len, t)
