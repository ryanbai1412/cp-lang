// expect: ok, output "YES NO\nYES\n4\n4\n3\n3\n3 4 5"
// Every `SortedSet` journey from std-api/R13 is one short call.
var set = SortedSet.of([1, 3, 5, 7])
// journey: add returning whether it was new
print(yes(set.add(4)), yes(set.add(4)))
// journey: remove one copy
print(yes(set.remove(7)))
// journey: predecessor
print(set.range(..5).last!)
// journey: successor
print(set.first_true(y => y > 3)!)
// journey: count in a range
print(set.range(2..=5).len)
// journey: k-th smallest
print(set.nth(1)!)
// journey: walk a range in order
print(set.range(2..).collect())
