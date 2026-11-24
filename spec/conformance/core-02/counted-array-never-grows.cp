// expect: E0203
// rule: 02.8.2
// input: "1 2 3\n"
// A counted array `[n]T` passes as a `List<T>` but never grows: it has no `push` (rebuild round 17 H11).
var a = read<[3]Int>()
a.push(4) // <- here
print(a.len)
