// expect: E0902
// rule: std.fenwick.Fenwick.kth
domain D = 0..3
var f = Fenwick<D, Int>()
f.add(D(0), 1)
print(yes(f.kth(0) == None)) // <- here
