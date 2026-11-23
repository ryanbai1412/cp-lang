// CF 1286A Garland: dp[used evens, parity of last] = fewest parity changes so far.
let n: 1..=100 = read()
let p = read<[n]0..=n>()
let free_even = n / 2 - p.count(x => x > 0 and x % 2 == 0)
fn allowed(x: Int, r: Int) -> Bool = x == 0 or x % 2 == r
fn uses(x: Int, r: Int) -> Int = if x == 0 and r == 0: 1 else: 0
var dp: [0..=free_even, 0..2]?Int = None
for r in 0..2:
    if allowed(p[0], r) and uses(p[0], r) <= free_even: dp[uses(p[0], r), r] = 0
for x in p[1..]:
    var next: [0..=free_even, 0..2]?Int = None
    for u in 0..=free_even:
        for q in 0..2:
            if let d = dp[u, q]:
                for r in 0..2:
                    let u2 = u + uses(x, r)
                    if allowed(x, r) and u2 <= free_even:
                        next[u2, r] min= d + (if q != r: 1 else: 0)
    dp = next
print(dp[free_even].present().min()!)
