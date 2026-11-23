// `[r]T` and `domain D = r` for a `Range<Int>` value `r` (R04b.1.5, R07.4.3).
let s = [5, 6, 7, 8]
let w = s[2..]
let r = 1..3
var t: [r]Int = 0
t[2] = 9
print(t[2], t.len)
let u = [w.keys] k => s[k] * 10
print(u, u[3])
domain D = w.keys
print(D.count, Int(D.first!))
