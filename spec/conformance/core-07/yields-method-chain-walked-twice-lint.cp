// expect: L0602
// rule: 07.1.3 07.2.9
fn doubled(self: List<Int>) -> yields Int:
    for x in self: yield 2 * x
let a = [1, 2, 3]
let c = a.doubled().take(2)
print(c.sum())
print(c.sum()) // <- here
