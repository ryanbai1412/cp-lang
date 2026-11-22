// expect: R0401 at 6 (checked)
// rule: std.fenwick.Fenwick.fold
domain P = 0..5
var fw = Fenwick<P, Int>([P] i => i.ord + 1)
print(fw.fold(P(3)..P(1)))
