// expect: E0301 at 4
// rule: 04b.1.7 04a.11.3
let s = [1, 2, 3, 4, 5]
let t = s[1..].rebase
let u = s[2..].rebase
print(t[u.Idx(0)])
