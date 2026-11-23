// Synthesis #3 Leap 3: `D.nth(i)` traps on out-of-range ordinal (both cpc and cpi).
let n: 1..=10 = read()
domain V = 1..=n
let bad: V = V.nth(n)
print(Int(bad))
