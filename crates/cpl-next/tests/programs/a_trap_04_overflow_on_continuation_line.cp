// expect: R0202
// rule: R09.3
// input: "4611686018427387904\n"
// A multi-line expression: the failing `+` is on line 6.
let n: Int = read()
let s = (n - 1) +
    (n + n)  // <- here
print(s)
