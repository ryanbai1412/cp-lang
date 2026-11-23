// expect: ok, output "1 3 a b 2 7"
// The prelude brings in std.algebra's body: no compiler special case.
let (lo, hi) = minmax(7, 2)
print(minmax(3, 1), minmax("b", "a"), lo, hi)
