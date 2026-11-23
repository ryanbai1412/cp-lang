// expect: ok, output "1\n2"
// input: "2\n1\n2\n"
// rule: 01.9.1
// An enum below `@!multitest` may use the per-test domain declared above it.
@!multitest
let n = read<1..=10>()
domain V = 1..=n
enum E = A(v: V) | B
let e = E.A(V.last!)
print(n)
