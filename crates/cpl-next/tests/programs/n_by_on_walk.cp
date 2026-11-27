// expect: ok, output "3\n5 1\n9 6 3 0\n2 1\n8 5 2"
// rule: R01.8.4
// `by` steps a domain or a reversed walk through the walk's impl of the `@operator("by")`
// trait, not only a range literal, and the result is a range again: it has a `len` and walks
// either way.
domain V = 0..6
print((V by 2).len)
var t: List<Int> = []
for v in V.rev by 4: t.push(Int(v))
print(t)
let r = (0..10).rev by 3
print(List(r))
fn evens(n: Int) -> Range<Int> = (0..n).rev by 2
print(evens(3).len, evens(1).len)
print(List((0..9).rev by 3))
