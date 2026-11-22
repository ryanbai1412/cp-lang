// expect: ok, output "1 9"
// rule: 02.5.3
let a = (to: 1, w: 2)
let b = a with w: 9
print(b.to, b.w)
