// expect: ok, output "12\n4"
// rule: 04a.2.3 04a.11.1
let p = [1, 2, 3, 4, 5]
let q = p[..2] + p[3..]
print(q.sum())
print(q.len)
