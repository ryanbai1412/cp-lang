// expect: ok, output "1 9 0 5"
// rule: 01.8.3
let a = (lo: 1, hi: 5)
let b = a with hi: 9
let c = (..a, lo: 0)
print(b.lo, b.hi, c.lo, c.hi)
