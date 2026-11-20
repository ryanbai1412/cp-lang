// Codeforces 2237 D: count subarrays whose digit+1 sum is nonzero mod 3,
// subtracting alternating pairs via the current alternating run length.
@!multitest
let n: Int = read()
let s: Str = read()
var ans = 0
var cnt: [0..3]Int = 0
var pref = 0
var cur_len = 0
cnt[0] = 1
for i in 0..n:
    pref = (pref + (s[i] - '0') + 1) % 3
    if i == 0 or s[i] == s[i - 1]: cur_len = 1
    else: cur_len += 1
    ans += i + 1 - cnt[pref] - (cur_len - 1) / 2
    cnt[pref] += 1
print(ans)
