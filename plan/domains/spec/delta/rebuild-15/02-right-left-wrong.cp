// CF 2000D Right Left Wrong
// in: t; per test n, a_1..a_n, s of 'L'/'R'. out: the maximum score.
@!multitest
let n = read<2..=2e5>()
let a = read<[n]1..=1e5>()
let s: Str = read()
let pre = a.scan(+)
var l = 0
var r = n - 1
var score = 0
while true:
    while l < n and s[l] != 'L': l += 1
    while r >= 0 and s[r] != 'R': r -= 1
    if l >= r: break
    score += pre.fold(l..=r)
    l += 1
    r -= 1
print(score)
