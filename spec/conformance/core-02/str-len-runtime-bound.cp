// expect: ok, output "3"
// rule: 02.3.2
// input: "3\nabc\n"
// `Str(len: n..=n)` with a read `n` is a string of exactly `n` characters, checked on read.
let n: 1..=10 = read()
let s: Str(len: n..=n) = read()
print(s.len)
