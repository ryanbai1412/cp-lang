// expect: ok, output "6\n7"
// rule: 04b.1.9
let s = [5, 6, 7, 8]
let w = s[1..3]
for k in 0..w.len: print(w.nth(k))
