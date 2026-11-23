// Codeforces 2237 D, tuned: port.cp's count with `pref` declared `0..3` and
// the weight written as 1 or 2, so `% 3` has a non-negative operand and is
// C++'s `%` rather than a Euclidean remainder.

@!multitest
let n: 1..=1_000_000 = read()
let s: [0..n]Char = read()
var ans = 0
var cnt: [0..3]Int = [1, 0, 0]
var pref: 0..3 = 0
var cur_len = 0
for (i, c) in s.items:
    pref = (pref + (if c == '1': 2 else: 1)) % 3
    cur_len = if i == 0 or c == s[i - 1]: 1 else: cur_len + 1
    ans += i + 1 - cnt[pref] - (cur_len - 1) / 2
    cnt[pref] += 1
print(ans)
