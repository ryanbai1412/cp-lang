// here: 5
// `a.last = 5` on a `[D]T`: "no field `last` to assign to", with no fix.
domain D = 0..3
var a: [D]Int = [1, 2, 3]
a.last = 5
print(a)
