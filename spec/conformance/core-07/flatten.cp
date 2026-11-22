// expect: ok, output "6 6"
// rule: 07.12.2
print([[1, 2], [3]].flatten().sum(), [1, 2].flat_map(x => [x, x]).sum())
