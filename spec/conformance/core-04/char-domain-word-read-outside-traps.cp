// expect: R0102
// rule: 04.2.24
// input: "abz\n"
// A character outside the letter domain stops the read.
domain Letter = 'a'..='c'
let s: [domain S]Letter = read()
print(s.len)
