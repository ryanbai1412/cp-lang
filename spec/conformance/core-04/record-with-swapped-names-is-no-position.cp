// expect: E0302 at 8
// rule: 04.2.10
// A record names the factors it fills, so one listing them out of order is no position, even when both factors have the same type.
domain C = 0..3
domain State = (lo: C, hi: C)
let x: C = C.first!
let y: C = C.last!
let s: State = (hi: y, lo: x)
print(s.lo.ord)
