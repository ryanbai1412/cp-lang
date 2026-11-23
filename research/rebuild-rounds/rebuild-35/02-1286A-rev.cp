// CF 1286A Garland, revised: the bulbs hang in a circle, so the last bulb also neighbours
// the first. dp[used evens, parity of first, parity of last].
let n: 1..=100 = read()
let p = read<[n]0..=n>()
let free_even = n / 2 - p.count(x => x > 0 and x % 2 == 0)
fn allowed(x: Int, r: Int) -> Bool = x == 0 or x % 2 == r
fn uses(x: Int, r: Int) -> Int = if x == 0 and r == 0: 1 else: 0
var dp: [0..=free_even, 0..2, 0..2]?Int = None
for r in 0..2:
    if allowed(p[0], r) and uses(p[0], r) <= free_even: dp[uses(p[0], r), r, r] = 0
for x in p[1..]:
    var next: [0..=free_even, 0..2, 0..2]?Int = None
    for u in 0..=free_even:
        for f in 0..2:
            for q in 0..2:
                if let d = dp[u, f, q]:
                    for r in 0..2:
                        let u2 = u + uses(x, r)
                        if allowed(x, r) and u2 <= free_even:
                            next[u2, f, r] min= d + (if q != r: 1 else: 0)
    dp = next
var best: ?Int = None
for f in 0..2:
    for q in 0..2:
        if let d = dp[free_even, f, q]: best min= d + (if n > 1 and f != q: 1 else: 0)
print(best!)
