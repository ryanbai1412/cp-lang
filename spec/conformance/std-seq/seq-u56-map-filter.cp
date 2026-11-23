// expect: ok, output "4 6 20 30"
// Acceptance for seq/U56 (map, filter, filter_map), handbook/product/std/sequences-usecases.md.
print([1, 2, 3].map(x => x * 2).filter(x => x > 2).collect(), [1, 2, 3].filter_map(x => if x > 1: Some(x * 10) else: None).collect())
