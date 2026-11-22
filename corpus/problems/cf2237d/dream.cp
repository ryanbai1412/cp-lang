// Codeforces 2237 D — Binary Substring Reductions
//
// Assign weight `1` to `'0'` and `2` to `'1'` (i.e. `(c - '0') + 1`). A
// non-empty binary substring is valid iff:
// 1. Its weight sum is nonzero mod 3. Maintaining `cnt[r]` for prefix sums
//    mod 3 (seeded with `cnt = [1, 0, 0]` for the empty prefix of sum 0),
//    `i + 1 - cnt[pref]` counts all substrings ending at `i` with nonzero sum
//    mod 3.
// 2. It is not an alternating string (`010...` or `101...`) of length `>= 2`.
//    Every even-length alternating string has pairs `(0, 1)` of weight `1 + 2
//    = 3 == 0 (mod 3)`, so it is already excluded by `cnt[pref]`. Each
//    odd-length alternating suffix of length `3, 5, ..., <= cur_len` has
//    nonzero weight mod 3 and must be subtracted once; there are exactly
//    `(cur_len - 1) / 2` such suffixes ending at `i`, where `cur_len` is the
//    length of the maximal alternating run ending at `i`.

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()
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
