// expect: E0504
// rule: 02.7.3
// has: "not in the design"
// A payload-free variant's `Int` is its position; other numbers come from an `Into<Int>` impl.
enum Color:
    Red = 2 // <- here
    Green
print(Int(Color.Red))
