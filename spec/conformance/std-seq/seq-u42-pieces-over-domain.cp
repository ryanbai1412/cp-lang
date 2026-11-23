// expect: ok, output "1\n3\n5\n0 3\n1 2\n2 1\n3 0"
// Acceptance for seq/U42 (windows and pairs over a domain array), handbook/product/std/sequences-usecases.md.
domain I = 0..4
let a: [I]Int = [I] i => Int(i) * Int(i)
for (x, y) in a.adjacent():
    print(y - x)
for (i, j) in zip(I, I.rev):
    print(i, j)
