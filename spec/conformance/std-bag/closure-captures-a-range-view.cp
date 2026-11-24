// expect: ok, output "2\n2"
// rule: 05.2.1 std.bag.SortedBag.range
// A closure may capture a bag's range view and walk it; the view keeps its type inside the closure (adversarial round 46).
var b = SortedBag.of([1, 3])
let v = b.range(..)
print(v.len)
let f = () => [x for x in v].len
print(f())
