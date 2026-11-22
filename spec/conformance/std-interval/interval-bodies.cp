// expect: ok, output "2 1 2\n1 2 1\n2 3 2\n2\n1\n2 1 YES\n3\n0\n1 4\n2 3 YES NO YES NO\nYES YES\n3 YES 2 7\nYES 2\n1 2\n3 10"
use std.interval.{IntervalMap, IntervalSet}
domain D = 0..3
let a: [D]Int = [D] v => [1, 1, 2][Int(v)]
var m = IntervalMap<D, Int>.from(a)
print(m.runs, m.get(D(1)), m.get(D(2)))
for p in m.range(D(1)..): print(Int(p.range.start.ord), Int(p.range.end.ord), p.val)
for p in m.each_rev(): print(p.val)
print(Int(m.first_where(D.cuts.nth(0), v => v == 2)!), Int(m.first_where(D.cuts.nth(1), v => v == 1)!), m.first_where(D.cuts.nth(3), v => true) == None)
m.set(D(1), 3)
print(m.runs)
domain T = 0..10
var s = IntervalSet<T>()
print(s.covered)
s.add(T(1)..T(3))
s.add(T(3)..T(5))
print(s.runs, s.covered)
s.remove(T(2)..T(3))
print(s.runs, s.covered, s.contains(T(1)), s.contains(T(2)), s.contains_range(T(3)..T(5)), s.contains_range(T(1)..T(4)))
print(s.run_at(T(4)) == Some(T(3)..T(5)), s.run_at(T(2)) == None)
print(Int(s.next_covered(T(2))!), s.next_covered(T(5)) == None, Int(s.next_gap(T(1))!), Int(s.next_gap(T(7))!))
s.add(T(5)..)
print(s.next_gap(T(6)) == None, s.runs)
for r in s.each(): print(Int(r.start.ord), Int(r.end.ord))
