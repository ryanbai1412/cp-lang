// expect: ok, output "YES\nYES NO\n3\n0 1 2\n3"
// Every `Dsu` journey from std-api/R13 is one short call.
var d = Dsu<0..4>()
// journey: union
print(yes(d.unite(0, 1)))
d.unite(1, 2)
// journey: same set
print(yes(d.same(0, 2)), yes(d.same(0, 3)))
// journey: set size
print(d.size(2))
// journey: list the sets
print(d.groups())
