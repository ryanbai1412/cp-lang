// expect: ok, output "YES"
// rule: std.fenwick.Fenwick.first_ge
domain D = 0..3
var f = Fenwick<D, Int>()
for v in D: f.add(v, Int(v) + 1)
print(yes(f.first_ge(6) == Some(D.cuts.last!) and f.first_ge(7) == None))
