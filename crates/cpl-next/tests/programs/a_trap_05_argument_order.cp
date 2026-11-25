// expect: R0203
// rule: R06 arguments evaluate left to right
// Arguments are evaluated left to right: the division by zero comes before the overflow.
fn pair(a: Int, b: Int) -> Int = a + b
let z = 0
print(pair(1 / z, 9223372036854775807 + 1))  // <- here
