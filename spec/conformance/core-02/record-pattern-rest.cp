// expect: ok, output "1"
// rule: 02.5.5
// `..` stands for the fields a record pattern leaves out.
let e = (u: 1, v: 2, w: 9)
let (u:, ..) = e
print(u)
