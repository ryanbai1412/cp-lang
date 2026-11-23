// expect: ok, output "1 1 2 2 1 2 3 1 2 3"
// Acceptance for seq/U22 (flat map, flatten and concat), handbook/product/std/sequences-usecases.md.
print([1, 2].flat_map(x => [x, x]).collect(), [[1], [2, 3]].flatten().collect(), [1, 2] + [3])
