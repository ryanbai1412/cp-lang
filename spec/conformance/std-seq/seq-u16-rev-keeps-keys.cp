// expect: ok, output "2 3\n1 2\n0 1"
// Acceptance for seq/U16 (walk backwards and keep the keys), handbook/product/std/sequences-usecases.md.
domain I = 0..3
let a: [I]Int = [1, 2, 3]
for (i, x) in a.rev.items:
    print(i, x)
