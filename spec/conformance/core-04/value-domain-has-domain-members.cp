// expect: ok, output "1 3 2"
// rule: 04b.1.7
// A value's own domain `w.Idx` has every domain member: `wrap`, `count`, `nth`.
struct Wrap(xs: List<Int>):
    domain Idx = 0..xs.len
let w = Wrap([5, 6, 7])
print(w.Idx.wrap(4).ord, w.Idx.count, w.Idx.nth(2).ord)
