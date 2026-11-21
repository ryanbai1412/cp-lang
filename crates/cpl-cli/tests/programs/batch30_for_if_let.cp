let dp: [0..4]?Int = [some(4), none, some(6), some(9)]
for p in 0..4 if let cur = dp[p]:
    print(p, cur)
for p in 0..4 if let cur = dp[p], cur > 4:
    print("big", p, cur)
for p in 0..4 if let cur = dp[p], p > 0, let nxt = dp[(p + 1) % 4], cur > nxt or p == 2:
    print("pair", p, cur, nxt)
for p in 0..4 if p % 2 == 1: print("odd", p)
for p in 0..4 if p > 0, let cur = dp[p]: print("late", p, cur)
var seen = 0
let ws = [5, 6, 7, 8]
'outer: for i in 0..4 if let cur = dp[i] and ws[i] != 7:
    for j in 0..3 if j != 1:
        if j == 2 and i == 3: break 'outer
        seen += cur + j
    if i == 0: continue
    print("after", i)
print(seen)
fn pick(xs: List<?Int>) -> Int:
    for x in xs if let v = x, v % 2 == 0: return v
    -1
print(pick([some(3), none, some(8)]), pick([none]))
