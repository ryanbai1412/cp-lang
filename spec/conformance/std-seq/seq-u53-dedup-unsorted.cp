// expect: ok, output "3 1 2 1 2 1"
// Acceptance for seq/U53 (distinct values in first-seen order, and adjacent dedup), handbook/product/std/sequences-usecases.md.
print([3, 1, 3, 2, 1].distinct().collect(), [1, 1, 2, 1].deduped())
