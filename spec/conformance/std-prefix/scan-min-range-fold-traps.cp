// expect: R0401 at 7
// A `scan(min)` table answers prefix reads, but a range fold would subtract
// two prefix minima and print garbage, so it traps instead (seq/U7).
let a = [3, 1, 4, 1, 5]
let p = a.scan(min)
print(p.prefix(2), p.all)
print(a.scan(+).fold(1..3))
print(p.fold(2..5))
