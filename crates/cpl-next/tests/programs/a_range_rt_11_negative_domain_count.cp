// expect: ok, output "0"
// rule: spec 4
// input: "-3\n"
// domain with a negative run-time count is empty (Spec 4), arrays over it have no cells.
let n: Int = read()
domain V = 0..n
var a: [V]Int = 0
var c = 0
for v in V: c += 1
print(c)
