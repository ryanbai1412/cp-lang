// expect: ok, output "8 5 6"
// rule: 04a.9.5
// A filter is a Seq with no keys: iterate it, in order.
let xs = [3, 8, 5, 6]
var out: List<Int> = []
for x in xs.filter(x => x > 4): out.push(x)
print(out)
