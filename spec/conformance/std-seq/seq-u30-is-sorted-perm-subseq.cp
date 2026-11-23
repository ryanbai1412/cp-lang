// expect: ok, output "YES YES"
// Acceptance for seq/U30 (is it sorted, a permutation, a subsequence), handbook/product/std/sequences-usecases.md.
print(yes([1, 2, 2].is_sorted()), yes([1, 3].is_subsequence([1, 2, 3])))
