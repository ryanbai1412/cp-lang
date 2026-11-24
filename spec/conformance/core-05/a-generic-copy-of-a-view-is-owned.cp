// expect: ok, output "1 2 3 4 50 3 9"
// rule: 05.2.2 05.7.8
// A fn not declared `lends` returns no lent value, so a `-> T` whose `T` binds a slice view gives the owned `List`; `push` is the list's own, not std's `Counter.push`.
fn dup<T>(self: T) -> T = copy(self)
let a = [1, 2, 3, 4]
let w = a[1..3]
var b = dup(w)
b[0] = 50
b.push(9)
print(a, b)
