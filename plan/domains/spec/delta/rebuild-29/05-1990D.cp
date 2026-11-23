// CF 1990D Grid Puzzle: whiten the grid with 2x2 squares and whole rows.
// Only 2x2 squares on columns 1-2 or 3-4 ever help. dp[s]: fewest operations for the rows
// done so far, where bit 0 / bit 1 of s says squares from the previous row already cover
// columns 1-2 / 3-4 of this row.

@!multitest
let n: 1..=2e5 = read()
let a = read<[n]0..=n>()
var dp: [4]?Int = None
dp[0] = Some(0)
for i in 0..n:
    var nx: [4]?Int = None
    for s in 0..4:
        let Some(cur) = dp[s] else: continue
        let need = (if a[i] >= 1 and s & 1 == 0: 1 else: 0) | (if a[i] >= 3 and s & 2 == 0: 2 else: 0)
        for t in 0..4:                 // t: squares started on this row, reaching the next
            if a[i] <= 4 and t & need == need: nx[t] min= cur + popcount(t)
            nx[t] min= cur + 1 + popcount(t)
    dp = nx
print(dp[0]!)
