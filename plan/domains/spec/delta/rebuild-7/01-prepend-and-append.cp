// Codeforces 1791C Prepend and Append
// in: t; per test n, then binary string s of length n. out: per test the shortest possible original length.
@!multitest
let n: 1..=2000 = read()
let s = read<Str of "01">()
var l = 0
var r = n
while r - l >= 2 and s[l] != s[r - 1]:
    l += 1
    r -= 1
print(r - l)
