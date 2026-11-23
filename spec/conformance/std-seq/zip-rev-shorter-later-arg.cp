// expect: ok, output "1-20 2-10\n6 6"
// rule: 07.6.4
// A later `zip` argument may be a `.rev` of a different length; pairs stop at the shorter and are read by walk position.
let a = [1, 2, 3]
let b = [10, 20]
print(["{x}-{y}" for (x, y) in zip(a, b.rev)])
print([x * y for (x, y) in zip(a.drop(1), a.rev)])
