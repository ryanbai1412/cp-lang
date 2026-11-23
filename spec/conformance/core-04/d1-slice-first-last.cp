// expect: ok, output "7 8"
// rule: 04a.5.1
// The first and last of a slice are `first` and `last`, not `w[0]` and `w[w.len - 1]`.
let s = [5, 6, 7, 8]
let w = s[2..4]
print(w.first!, w.last!)
