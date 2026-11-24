// expect: ok, output "1\n1\n2\n2\n4"
// rule: std.seq.flat_map
// `flat_map` yields the elements of what its closure gives back, over a domain as over a list.
domain M = [2]
let xs = List(M.flat_map(m => [(at: 1, m:), (at: 2, m:)])).sorted()
for c in xs: print(c.at)
let ys = List([1, 2].flat_map(n => [(at: n, k: 0), (at: n, k: 1)]))
print(ys.len)
