// expect: R0101 at 4
// input: "5"
// rule: 07.4.5
// A slice past the end of a `Str` is R0101 too.
let s = "abc"
let r: Int = read()
print(s[0..r])
