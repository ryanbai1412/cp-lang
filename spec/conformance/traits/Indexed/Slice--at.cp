// expect: ok, output "4\n1\n5"
let a = [3, 1, 4, 1, 5]
let s = a[2..]
for k in s.keys: print(s[k])
