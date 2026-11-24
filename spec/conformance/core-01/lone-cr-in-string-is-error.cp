// expect: E0107
// rule: 01.1.4
// A raw `\r` inside a string is an error too; the string writes the escape `\r`.
let s = "ab" // <- here
print(s.len)
