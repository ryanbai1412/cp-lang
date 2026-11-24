// expect: ok, output "6 5 7"
// rule: std.algebra.Comm
// tool: build -o -
// lacks: "std::function"
// A `Comm` brings its own monoid, so `Monoid.of` takes it by `monoid()` with its stored op
// rather than rebuilding one from its laws around a capture of the `Comm`; a capturing op
// would make every `Monoid<Int>` op a dynamic fn object.
let plus = Monoid(0, (a, b) => a + b).commutative()
let sums = SegTree(plus, [1, 2, 3])
let maxs = SegTree(max, [4, 1, 5])
print(sums.fold(0..3), maxs.fold(0..3), Monoid<Int>.of(plus).op(3, 4))
