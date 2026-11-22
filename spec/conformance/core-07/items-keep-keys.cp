// expect: ok, output "1 6\n2 7"
// rule: 07.2.3 07.3.2 07.4.8
let s = [5, 6, 7, 8]
for (i, x) in s[1..3].items: print(i, x)
