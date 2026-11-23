// Codeforces 1978C Manhattan Permutations
// in: t; per case n k. out: per case "No", or "Yes" then a permutation with Manhattan value k
with io(answers: ("Yes", "No"))
@!multitest
let (n, k) = read<(1..=2e5, 0..=1e12)>()
if k % 2 == 1 or k > n * n / 2:
    print(yes(false))
    return
var p = [i for i in 1..=n]
var (l, r) = (0, n - 1)
var left = k
while left > 0:
    let d = 2 * (r - l)
    if d <= left:
        (p[l], p[r]) = (p[r], p[l])
        left -= d
        l += 1
        r -= 1
    else:
        let j = l + left / 2
        (p[l], p[j]) = (p[j], p[l])
        left = 0
print(yes(true))
print(p)
