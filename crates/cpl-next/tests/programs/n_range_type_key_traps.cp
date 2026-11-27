// expect: R0101
// rule: R04.3.4
// An `Int` key past a range-type-keyed array's end is a key outside the array.
fn mk<D: Domain>() -> [D]Int = [D] d => d.ord * 10
let a = mk<2..=4>()
var i = 4
print(a[i])
i += 1
print(a[i])  // <- here
