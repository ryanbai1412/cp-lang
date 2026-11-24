// expect: ok, output "2\n1 2\n2 none\n2 2\n1 2\n1\n2 1"
// input: "3\n2\n"
// rule: 10.2.2
// Under `io(base: 1)` a position prints shifted inside a list of lists, an optional in a list, a tuple holding a list, a reversed list, a slice and a record holding a list of optionals.
with io(base: 1)
domain V = read(1..=5)
let x: V = read()
let l: List<V> = [x, V(0)]
print([[x], [V(0), x]])
print([Some(x), None])
print((x, [x]))
print(l.rev)
print(l[1..])
print([(a: x, b: [Some(V(0))])])
