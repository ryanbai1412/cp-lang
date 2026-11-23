// Codeforces 1986E Beautiful Array: fewest +k operations to make a shuffled array a palindrome.
// In: t; per test `n k`, then a_1..a_n. Out: the minimum, or -1, per test.
@!multitest
let n = read<1..=1e5>()
let k = read<1..=1e9>()
let a = read<[n]1..=1e9>()

// Only values with the same residue mod k can be paired; sorted, neighbours pair up.
var odd = 0
var total = 0
for run in a.sorted(by: x => (x % k, x)).runs(by: x => x % k):
    let g = List(run)
    let m = g.len
    let step = (j: Int) => (g[j + 1] - g[j]) / k
    if m % 2 == 0:
        total += sum(step(j) for j in 0..m by 2)
    else:
        odd += 1
        // Skip g[0] first, then slide the skipped element right two at a time.
        var cur = sum(step(j) for j in 1..m by 2)
        var best = cur
        for i in 0..m - 1 by 2:
            cur += step(i) - step(i + 1)
            best min= cur
        total += best
print(if odd > n % 2: -1 else: total)
