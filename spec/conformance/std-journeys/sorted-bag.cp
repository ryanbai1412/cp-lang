// expect: ok, output "YES\n3\n5\n3\n3\n3 3 5"
// Every `SortedBag` journey from std-api/R13 is one short call.
var bag = SortedBag.of([1, 3, 3, 5, 7])
// journey: add
bag.add(4)
// journey: remove one copy
print(yes(bag.remove(7)))
// journey: remove at a cursor
bag.remove_at(bag.cursor_ge(4)!)
// journey: predecessor
print(bag.range(..5).last!)
// journey: successor
print(bag.first_true(y => y > 3)!)
// journey: count in a range
print(bag.range(2..=5).len)
// journey: k-th smallest
print(bag.nth(1)!)
// journey: walk a range in order
print(bag.range(2..).collect())
