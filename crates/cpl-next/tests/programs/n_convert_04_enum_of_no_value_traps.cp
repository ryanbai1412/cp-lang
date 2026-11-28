// expect: R0102
// rule: R02.7
// input: "1"
// `E(x)` traps where no variant is at position `x`, in both engines.
enum Color:
    Red
    Green
let k = read<Int>()
print(Int(Color(k - 1)))
print(Int(Color(k + 1))) // <- here
