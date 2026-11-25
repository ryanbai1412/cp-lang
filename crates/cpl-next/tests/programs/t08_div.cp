// expect: R0203
// input: "7 2\n"
let a: Int = read()
let b: Int = read()
print("{a / b} {a % b} {(0 - a) / b} {(0 - a) % b} {a / (0 - b)} {a % (0 - b)}")
print(a / (b - b))  // <- here
