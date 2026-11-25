// expect: R0401
// rule: R09.3: the message is "n must be small: 7" (the harness cannot see it)
// input: "7\n"
let n: Int = read()
assert n < 5, "n must be small: {n}"  // <- here
print(n)
