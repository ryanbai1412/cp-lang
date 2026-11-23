// expect: ok, output "0 5\n1 6\n1 3\n2 4"
// Acceptance for seq/U15 (walk two sequences together, or with positions), handbook/product/std/sequences-usecases.md.
for (i, x) in [5, 6].items:
    print(i, x)
for (x, y) in zip([1, 2], [3, 4]):
    print(x, y)
