// expect: ok, output "3 4"
// rule: 02.7.6
let r = (total: 3, take: 4)
let (total: t, take: k) = r
print(t, k)
