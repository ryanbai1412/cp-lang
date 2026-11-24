// expect: ok, output "1 4 3"
// rule: 10.3.8
// The way to print a `HashSet` is to choose its order: `s.sorted()` builds a list, in `print` or in a string.
var h: HashSet<Int> = HashSet()
h.add(4)
h.add(1)
let x: ?Int = Some(3)
print("{h.sorted()} {x ?? 0}")
