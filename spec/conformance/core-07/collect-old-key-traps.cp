// expect: R0101
// rule: 07.4.4
let s = [5, 6, 7, 8]
let c = s[2..].collect()
print(c[3]) // <- here
