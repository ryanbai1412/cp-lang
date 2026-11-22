// expect: ok, output "7 8 7"
// rule: 07.4.4
let s = [5, 6, 7, 8]
let c = s[2..].collect()
let l = List(s[2..])
print(c[0], c[1], l[0])
