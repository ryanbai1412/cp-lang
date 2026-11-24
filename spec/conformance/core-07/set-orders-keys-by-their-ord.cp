// expect: ok, output "3 1\n3 2 1\nYES NO\n2 1 2 3\n2 1\n0 1\n1 7\n1 5"
// rule: 07.11.1
// A set's order is its key type's `Ord`, a user one too, on both engines.
var s = Set<Desc<Int>>()
for x in [2, 3, 1]: s.insert(desc(x))
print(s.min()!.x, s.max()!.x)
print([d.x for d in s])
print(yes(desc(2) in s), yes(desc(7) in s))
print(s.first_ge(desc(2))!.x, s.first_gt(desc(2))!.x, s.last_le(desc(2))!.x, s.last_lt(desc(2))!.x)
s.remove(desc(3))
print([d.x for d in s])
var t = Set<(Int, Desc<Int>)>()
t.insert((1, desc(5)))
t.insert((1, desc(7)))
t.insert((0, desc(1)))
print([(a, d.x) for (a, d) in t])
