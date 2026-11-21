// Codeforces 2147 C: split into maximal alternating runs; any run starting
// with '1' of length 3 mod 4 makes the arrangement impossible.
@!multitest
let n: Int = read()
let s: Str = read()
let cuts = [0] + [i for i in 1..n if s[i] == s[i - 1]] + [n]
let ok = all(s[l] != '1' or (r - l) % 4 != 3 for (l, r) in cuts.adjacent)
print(if ok: "YES" else: "NO")
