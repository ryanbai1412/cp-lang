// expect: square_by_hand at 6
// rule: 06.4.9
/// The square of `x`.
@lint(square_by_hand, from: "x * x", to: "sq(x)")
pub fn sq(x: Int) -> Int:
    x * x
let a: Int = read()
print((a - 1) * (a - 1), a * (a + 1))
