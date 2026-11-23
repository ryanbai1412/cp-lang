// expect: ok, output "8 7\n-1"
// Acceptance for seq/U5 (transition point of a monotone predicate), handbook/product/std/sequences-usecases.md.
print((0..100).first_true(x => x * x >= 50)!, (0..100).last_true(x => x * x < 50)!)
print((0..10).first_true(x => x > 20) ?? -1)
