// CF 1343D Constant Palindrome Sum: for each target x, a pair costs 0 if it
// already sums to x, 1 if one change reaches x, else 2. Sweep x over 2..=2k.
@!multitest
let n: 2..=2e5 = read()
let k: 1..=2e5 = read()
domain X = 2..=2 * k
let a = read<[n]1..=k>()

var open: [X]Int = 0
var close: [X]Int = 0
var exact: [X]Int = 0
for (u, v) in a.zip(a.rev).take(n / 2):
    open[X(min(u, v) + 1)] += 1
    close[X(max(u, v) + k)] += 1
    exact[X(u + v)] += 1

var inside = 0
var best = n
for x in X:
    inside += open[x]
    best min= n - inside - exact[x]
    inside -= close[x]
print(best)
