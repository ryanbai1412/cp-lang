// expect: ok, output "7 8 2"
// rule: 04b.1.6
let s = [5, 6, 7, 8]
let w = s[2..]
let t = [w.keys] k => w[k]
print(t[2], t[3], t.len)
