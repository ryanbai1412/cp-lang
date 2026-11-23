// expect: ok, output "7"
// rule: 2.7.6
let r = (total: 3, take: 4)
match r:
    (total: t, take: k) => print(t + k)
