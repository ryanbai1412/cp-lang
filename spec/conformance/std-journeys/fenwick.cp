// expect: ok, output "8\n5\n2"
// Every `Fenwick` journey from std-api/R13 is one short call.
var f = Fenwick([1, 2, 3])
// journey: add at a point
f.add(0, 5)
// journey: prefix sum
print(f.prefix((0..2).r))
// journey: range sum
print(f.fold(1..3))
// journey: first prefix reaching a bound
print(f.first_ge(7)!)
