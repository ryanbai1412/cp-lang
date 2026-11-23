// expect: R0401 at 5 (checked)
// rule: std.fenwick.Fenwick.kth
domain D = 0..3
var f = Fenwick<D, Int>()
f.add(D(0), 1)
print(yes(f.kth(0) == None))
