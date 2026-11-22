// expect: ok, output "6 7"
// rule: 04b.1.10
let s = [5, 6, 7, 8]
let w = s[1..3]
let (i, j) = (s.len - 3, s.len - 2)
print(w[i], w[j])
