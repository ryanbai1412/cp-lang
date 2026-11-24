// expect: R0501 (checked)
// rule: 02.3.2
// input: "3\nab\n"
// A string shorter than its exact length traps on read.
let n: 1..=10 = read()
let s: Str(len: n..=n) = read() // <- here
print(s.len)
