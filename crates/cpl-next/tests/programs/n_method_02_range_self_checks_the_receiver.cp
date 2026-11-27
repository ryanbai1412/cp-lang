// expect: R0201
// rule: R03.1.4
// input: "-4\n"
// An `Int` receiver of a `self: 0..` fn is checked into the range, as an argument is.
@get fn half(self: 0..) -> Int = self / 2
let n: Int = read()
print(n.half) // <- here
