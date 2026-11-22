// expect: ok, output "6 7"
// rule: 04b.1.8
let s = [5, 6, 7, 8]
let w = s[1..3]
print(w.first!, w.nth(1))
