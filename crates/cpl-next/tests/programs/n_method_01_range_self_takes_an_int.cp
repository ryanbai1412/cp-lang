// expect: ok, output "8 49 2 3"
// rule: R06.4.2
// A fn whose `self` is a range type, here `0..`, extends an `Int` receiver: the receiver is
// checked into that range as an argument is, and `@get` reads it without `()`.
@get fn half(self: 0..) -> Int = self / 2
fn plus(self: 0..=9, k: Int) -> Int = self + k
let n: Int = 99
print(16.half, n.half, 5.half, 1.plus(2))
