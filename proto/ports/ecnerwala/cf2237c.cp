// Codeforces 2237 C — Running Total Minimization
//
// Processing `a` left-to-right, at each `x = a[i]` we minimize the running
// threshold `cur`: if `x >= cur`, resetting `cur = x` is strictly smaller than
// `cur + x` and dominates all earlier prefixes; if `x < cur`, we add `x`
// (`cur += x`), giving the minimum reachable final value greedily in O(n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
var cur = a[0]
for x in a[1..n]: cur = if x >= cur: x else: cur + x
print(cur)
