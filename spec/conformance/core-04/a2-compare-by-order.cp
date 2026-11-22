// expect: ok, output "eq\nstarts"
// rule: 04a.2.2 04a.11.1 04b.1.3
let s = "abcabc"
if s[0..3] == s[3..6]: print("eq")
if s[3..].starts_with(s[0..2]): print("starts")
