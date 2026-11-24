// expect: ok, output "2 2"
// Acceptance for seq/U52 (coordinate compression), handbook/product/std/sequences-usecases.md: `xs.compress()` and `compress(xs)` are one fn.
let (domain V, c) = [30, 10, 30].compress()
let (domain W, d) = compress([3, 1])
print(V.count, W.count)
