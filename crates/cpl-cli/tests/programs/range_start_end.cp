struct Seg(lo: Int, hi: Int)

let s = Seg(10, 20)
let r1 = 2..=4
let r2 = 2..4
print(r1.start, r1.end, r1.last!, r1.len)
print(r2.start, r2.end, r2.last!, r2.len)
print(s.lo, s.hi)

domain D = 1..=5
let rd = D(2)..=D(4)
print(rd.start.ord, rd.end.ord, rd.len)
