// expect: ok, output "3 2 1 3 2 1\n0 9\n1 5\n1 2"
// Acceptance for seq/U12 (sort or sorted by key, descending, stable), handbook/product/std/sequences-usecases.md.
var s = [3, 1, 2]
s.sort(by: x => -x)
print(s, [3, 1, 2].sorted(desc: true))
print([(1, 2), (1, 5), (0, 9)].sorted(by: p => (p.0, -p.1)))
