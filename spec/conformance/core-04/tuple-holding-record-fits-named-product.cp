// expect: ok, output "1"
// rule: 04.2.10
// A tuple variable whose field is a record fits a tuple type with a named product in that position, as the record alone fits the product.
domain C = 0..3
domain State = (lo: C, hi: C)
let x: C = C.first!
let t: ((lo: C, hi: C), Int) = ((lo: x, hi: x), 1)
let u: (State, Int) = t
print(u.1)
