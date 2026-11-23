// expect: ok, output "312"
// Acceptance for seq/U36 (a fold with no identity), handbook/product/std/sequences-usecases.md.
print([3, 1, 2].reduce((a, b) => a * 10 + b)!)
