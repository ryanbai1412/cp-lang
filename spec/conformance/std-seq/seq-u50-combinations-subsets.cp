// expect: ok, output "3 4 3 2"
// Acceptance for seq/U50 (combinations, subsequences and products), handbook/product/std/sequences-usecases.md: all four need no `use`.
print([1, 2, 3].combinations(2).count(), [1, 2].subsequences().count(), [1, 2, 3].pairs().count(), product([1, 2], "x").count())
