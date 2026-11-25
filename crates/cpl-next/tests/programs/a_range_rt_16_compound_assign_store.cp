// expect: R0201
// rule: R03.1.4
// input: "5\n"
// compound assignment into a ranged var is a store and is checked.
let n: Int = read()
var x: 0..n = 4
x += 1  // <- here
print(x)
