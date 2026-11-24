// expect: ok, output "3 3\n3 3\n1 1\n1 1\n4 4\n4 4\n3 3 1 1 4 4"
// rule: 08.2.2
// `Zipped<A, B, T, U>`'s `U` is not `map`'s or `flat_map`'s: each call's generics are its own.
let xs = [3, 1, 4]
print(List(zip(xs, xs).map(x => [x, x])))
print(List(zip(xs, xs).flat_map(p => [p.0, p.1])))
