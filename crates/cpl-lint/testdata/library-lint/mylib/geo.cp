/// The square of `x`.
@lint(square_by_hand, from: "x * x", to: "sq(x)")
pub fn sq(x: Int) -> Int:
    x * x
