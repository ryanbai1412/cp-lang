// expect: ok
// tool: fix
// rewritten: "let xs = [1, 2]\nprint(yes(xs.contains(1)))"
// `in` on a list is an error, and a lone `x in xs` carries the fix `xs.contains(x)`, which scans in O(n) where it shows.
let xs = [1, 2]
print(yes(1 in xs))
