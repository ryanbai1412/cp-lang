// expect: R0102
// rule: R02.7
// input: "3"
// `E(x)` traps where no variant has the value `x`, in both engines.
enum Color:
    Red = 2
    Green = 5
let k = read<Int>()
print(Int(Color(k + 2)))
print(Int(Color(k))) // <- here
