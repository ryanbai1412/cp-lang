// expect: ok, output "5 5 5 5\n0 1 2 3"
// rule: 07.6.4
// `zip`'s later arguments may be a `.rev`, read by walk position, while the first keys the pairs: `zip(a, a.rev)` pairs each element with its mirror.
let a = [1, 2, 3, 4]
print([x + y for (x, y) in zip(a, a.rev)])
print([k for (k, p) in zip(a, a.rev).items])
