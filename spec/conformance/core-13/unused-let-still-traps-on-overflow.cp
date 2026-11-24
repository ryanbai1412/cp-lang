// expect: R0202 at 3 (checked)
// rule: 13.1.1 09.2.1
// input: "10000000000\n"
// A `let` whose value is never used still evaluates, so a checked build still traps on its overflow (adversarial round 46).
let x = read<Int>()
let unused = x * x
print("ok")
