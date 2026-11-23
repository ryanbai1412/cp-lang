// expect: ok, output "11 10\n1 1 1\n1 3 4 10\n0 1 3 5 8\n1 1 1"
// rule: std.ntt.convolve
// Bitwise `on:` ops go through their transforms and agree with the plain loop; `subset` is an
// `IndexOp`; a `min`-plus `shape: Both` merge agrees with the O(n m) loop.
use std.ntt.{subset}
let a = [3, 1, 4, 1, 5, 9, 2, 6]
let b = [2, 7, 1, 8, 2, 8, 1, 8]
print(convolve([1, 2], [3, 4], on: ^))
print(
    Int(convolve(a, b, on: ^) == convolve(a, b, on: (i, j) => i ^ j)),
    Int(convolve(a, b, on: |) == convolve(a, b, on: (i, j) => i | j)),
    Int(convolve(a, b, on: &) == convolve(a, b, on: (i, j) => i & j)),
)
print(convolve([1, 2, 3, 4], [1, 1, 1, 1], on: subset))
print(convolve([0, 1, 3], [0, 2, 5], over: Semiring(min, +), shape: Curved.Both))
let x = [0, 1, 3, 6, 10]
let y = [0, 0, 2, 5]
print(
    Int(convolve(x, y, over: Semiring(min, +), shape: Curved.Both) == convolve(x, y, over: Semiring(min, +), shape: Curved.Neither)),
    Int(convolve([5, 4, 2], [3, 3, 0], over: Semiring(max, +), shape: Curved.Both) == convolve([5, 4, 2], [3, 3, 0], over: Semiring(max, +), shape: Curved.Neither)),
    Int(convolve([1, 2, 3], [4, 5, 6], cyclic: true) == [31, 31, 28]),
)
