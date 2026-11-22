// expect: ok, output "7 8"
// rule: 12.4.13
let s = [5, 6, 7, 8]
let w = s[2..]
print(w.nth(0), w.nth(1))
