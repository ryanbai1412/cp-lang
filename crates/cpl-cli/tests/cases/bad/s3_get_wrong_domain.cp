// error: wrong domain
// Synthesis #3 Leap 3: `a.get(?I)` strictly enforces nominal domain safety (wrong domain rejected).
let n, m: 1..=10 = read()
domain R = 1..=n
domain C = 1..=m
let a: [R]Int = read()
let c: ?C = C.first
print(a.get(c) ?? 0)
