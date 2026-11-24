// expect: ok, output "15\n16\n36\n2"
// Every `SegTree` journey from std-api/R13 is one short call.
var s = SegTree(+, [1, 2, 3, 4, 5], act: add)
// journey: point set
s.set(0, 1)
// journey: point update
s.update(1, x => x * 1)
// journey: range fold
print(s.all)
s.set(4, 6)
print(s.fold(0..5))
// journey: range apply
s.apply(0..5, 4)
print(s.all)
// journey: first index where a prefix holds
print(SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, t => t <= 3))
