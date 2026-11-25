// expect: ok, output "3 4 2"
// rule: R04.2.8
// `V.at(x)` is the position with integer `x`, or `None` when `x` is outside `V`.
domain V = 2..=4
let a = V.at(3)
let b = V.at(5)
print(a!, b ?? V(4), V.at(-9) ?? V(2))
