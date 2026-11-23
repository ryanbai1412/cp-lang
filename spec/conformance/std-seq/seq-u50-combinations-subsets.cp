// expect: ok, output "3 4 3"
// Acceptance for seq/U50 (combinations, subsequences and products), handbook/product/std/sequences-usecases.md.
print([1, 2, 3].combinations(2).count(), [1, 2].subsequences().count(), [1, 2, 3].pairs().count())
