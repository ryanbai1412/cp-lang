// expect: R0101
// rule: R07.2.15
let r = 2..=10 by 4
print(r[6], r.nth(2), r.keys.len)
let e = (0..5).rev
print(e[4], e.nth(0))
print(r[4]) // <- here
