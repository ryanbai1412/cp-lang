// expect: ok, output "2 4\n0 1 2 3 4 5 6\n0 3 4 5 0"
// rule: std.range.shift
// One `shift` for cuts and positions: the elements at distances `r` after `self`, clipped.
domain V = 0..6
let c = V(2).r
let s = c.shift(1..=2)
print(s.len, s.first!.ord)
print([k.ord for k in c.shift(-5..=100)])
print(c.shift(10..12).len, [v.ord for v in V(4).shift(-1..=5)], V(0).shift(0..0).len)
