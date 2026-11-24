// expect: ok, output "YES YES YES YES NO"
// Acceptance for seq/U30 (is it sorted, a permutation, a subsequence), handbook/product/std/sequences-usecases.md.
let p = [2, 0, 1]
print(yes([1, 2, 2].is_sorted()), yes([1, 3].is_subsequence([1, 2, 3])), yes(p.is_permutation(p.keys)), yes([1, 2, 2].is_permutation([2, 1, 2])), yes([1, 2].is_permutation([1, 1])))
