// DMOPC 2021 C9 P1: x[i] + x[i+1] = s[i], all x positive integers. How many x[1] work?
// Each x[i] is ±x[1] + c; a + bounds x[1] below, a − bounds it above.
let s = read [read Int - 1]Int
let terms = s.scan((sign: 1, c: 0), (t, v) => (sign: -t.sign, c: v - t.c))
let lo = max(1, terms.filter(.sign > 0).max(of: t => 1 - t.c) ?? 1)
let hi = terms.filter(.sign < 0).min(of: t => t.c - 1) ?? inf
print(max(0, hi - lo + 1))
