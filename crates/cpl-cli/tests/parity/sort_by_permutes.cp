// sort(by:) with a key the sort works out once per element: merge runs of 7
// through a half-size buffer, then a permutation applied in place.
fn bucket(x: Int) -> Int: (x * 37) % 11
let xs = [(k * 7919) % 211 for k in 0..200]
var ys = xs.clone()
ys.sort(by: bucket)
print(ys)
var names = ["n{x % 13}" for x in xs]
names.sort(by: s => bucket(s.len))
print(names)
var ps = [(x % 5, x) for x in xs]
ps.sort(by: p => desc(bucket(p.1)))
print(ps)
