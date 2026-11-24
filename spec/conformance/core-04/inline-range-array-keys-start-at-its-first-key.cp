// expect: ok, output "10 20\n1 2\n10 20"
// rule: 04b.1.5
// An array over an inline range `[1..3]` has the range's own keys: `.keys` starts at 1, directly and through a generic `Indexed`.
fn firsts<S: Indexed<E, R>, E, R>(s: S) -> List<R>: [s[e] for e in s.keys]
let a: [1..3]Int = [10, 20]
print([a[e] for e in a.keys])
print(a.keys)
print(firsts(a))
