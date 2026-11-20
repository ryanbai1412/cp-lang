// Codeforces 2261 E: scan a balanced bracket string keeping the stack of open
// brackets. After each '(' find the deepest suffix of the stack whose span
// holds at most 2k other characters; those can be cleared to nest it fully.
@!multitest
let (n, k): (1..=2e5, 0..=2e5) = read()
let s: Str = read()
var open = [-1]
var ans = 0
for i in 0..n:
    if s[i] == ')':
        open.pop()
        continue
    open.push(i)
    let from = bsearch_first(0..open.len, j => (i + 1 - open[j]) - (open.len - j) <= 2 * k)!
    ans max= if from == 0: k + open.len - 1 else: open.len - from
print(min(ans, n / 2))
