// expect: ok, output "3\n5\n3"
// rule: 04b.1.7 04b.4.3 04a.11.3 04b.5.9
let s = [1, 2, 3, 4, 5]
let t = s[2..].rebase
print(t[t.Idx(0)])
print(t[t.Idx.wrap(-1)])
print(t.Idx.count)
