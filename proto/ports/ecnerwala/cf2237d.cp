// Codeforces 2237 D: count subarrays whose digit+1 sum is nonzero mod 3,
// subtracting alternating pairs via the current alternating run length.
@!multitest
let _n: Int = read()
let s: Str = read()
var ans = 0
var cnt: [0..3]Int = [1, 0, 0]
var pref = 0
var cur_len = 0
for (i, c) in s.items:
    pref = (pref + (c - '0') + 1) % 3
    cur_len = if i == 0 or c == s[i - 1]: 1 else: cur_len + 1
    ans += i + 1 - cnt[pref] - (cur_len - 1) / 2
    cnt[pref] += 1
print(ans)
