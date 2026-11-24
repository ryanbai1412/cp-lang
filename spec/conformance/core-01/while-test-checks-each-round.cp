// expect: ok, output "9"
// rule: 01.7.2
// A checked step inside a `while` test (here the cut `Town.cuts(a - 1)`) runs every time the
// test does, so its index follows `a` rather than keeping its first value.
domain Town = 1..=10
let xs = [Town.cuts] c => Int(c)
var a = 10
while a > 2 and xs[Town.cuts(a - 1)] >= 9: a -= 1
print(a)
