// `i..` is `i..=Int.last` and O(1) to build (R04, open-ended ranges), so a
// negative start builds; only a length or walk that doesn't fit traps.
var ok = 1..
ok = ok.clip(-3..)
print(ok.len)
let wide = -3..
print(wide.first!, wide.contains(5), wide.contains(-4))
var below = ..5
below = below.clip(-3..)
print(below.len)
for x in wide.take(3): print(x)
print(wide.drop(2).first!)
