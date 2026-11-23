// expect: ok, output "3\n3"
// rule: W8
// A `range` view taken from a value with no home (`SortedBag.of(xs).range(r)`) holds that value
// for as long as the view lives, instead of asking for a variable.
print(SortedBag.of([1, 3, 3, 7]).range(2..=7).len)
let r = SortedBag.of([1, 3, 3, 7]).range(2..=7)
print(r.len)
