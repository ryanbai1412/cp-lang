// expect: ok, output "3 20 0 5 9"
// rule: 01.4.5
let xs = [1, 2, 3]
let m = [1: 10, 2: 20]
let e: Map<Int, Int> = [:]
let t = (4, 5)
let r = (lo: 1, hi: 9)
print(xs[2], m[2], e.len, t.1, r.hi)
