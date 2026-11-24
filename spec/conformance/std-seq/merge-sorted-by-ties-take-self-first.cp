// expect: ok, output "a c b\n9 7 5 4 2"
// `merge_sorted` with `by:` takes ties from `self` first, and a descending key merges descending inputs.
let a = [(1, "a"), (2, "b")]
print([p.1 for p in a.merge_sorted([(1, "c")], by: p => p.0)])
print([9, 5, 2].merge_sorted([7, 4], by: x => -x))
