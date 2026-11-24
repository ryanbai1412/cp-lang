// here: 4
// C++'s `lower_bound(..) - a.begin()` (self-run mistakes): three errors, no fix.
var a: List<Int> = [1, 2, 2, 5]
print(lower_bound(a.begin(), a.end(), 2) - a.begin())
