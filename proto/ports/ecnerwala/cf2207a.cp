// Codeforces 2207 A: expand 101 -> 111 to get max ones, then shrink 111 -> 101
// left-to-right to get min ones.
@!multitest
let n: Int = read()
var s: Str = read()
for _ in 0..3:
    for i in 0..n - 2:
        if s[i] == '1' and s[i + 2] == '1': s[i + 1] = '1'
let max_cnt = count(s[i] == '1' for i in 0..n)
for i in 0..n - 2:
    if s[i] == '1' and s[i + 2] == '1': s[i + 1] = '0'
let min_cnt = count(s[i] == '1' for i in 0..n)
print(min_cnt, max_cnt)
