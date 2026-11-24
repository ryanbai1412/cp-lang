// expect: ok, output "1 2 3 3 4 3 2 1 1 2 3 3"
// Acceptance for seq/U54 (merge two sorted sequences), handbook/product/std/sequences-usecases.md. `by:` orders the merge; it used to be ignored.
print([1, 3].merge_sorted([2, 3]), [4, 1].merge_sorted([3, 2], by: x => -x), merge_sorted([1, 3], [2, 3]))
