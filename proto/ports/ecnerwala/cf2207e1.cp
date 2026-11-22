// Codeforces 2207 E1 — N-MEX Constructive (Easy Version)
//
// `a[i]` is the MEX of `b[0..=i]` (or suffix constraint): a valid `b[0..n]`
// exists iff `a` is non-increasing and `n - i - 1 <= a[i] <= n` for all `i`.
//
// Prepending `n` (`bounds = [n] + a`) gives the upper limit `bounds[i]` at each
// reverse step `i = n` down to `0`. Maintaining `next_val` = next unused value
// to push onto `stk`: by the lower bound `bounds[i] >= n - i`, at least
// `n - i + 1` values have been pushed while only `n - i` values have been
// popped so far, guaranteeing `stk.pop()!` is non-empty at every step.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()

if any(a[i] < a[i + 1] for i in 0..n - 1) or any(a[i] < n - i - 1 or a[i] > n for i in 0..n):
    print("NO")
    return

let bounds = [n] + a
var b: [0..n]Int = 0
var stk: List<Int> = []
var next_val = 0
for i in (0..=n).rev:
    let cur = bounds[i]
    while next_val <= cur:
        stk.push(next_val)
        next_val += 1
    let x = stk.pop()!
    if i < n: b[i] = x

print("YES")
print(b)
