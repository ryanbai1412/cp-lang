// Codeforces 2207 A: expand 101 -> 111 for the max ones, then shrink 111 -> 101
// left-to-right for the min ones.
@!multitest
let n: Int = read()
var s: Str = read()
for i in 0..n - 2 if s[i] == '1' and s[i + 2] == '1': s[i + 1] = '1'
let max_cnt = s.count('1')
for i in 0..n - 2 if s[i] == '1' and s[i + 2] == '1': s[i + 1] = '0'
print(s.count('1'), max_cnt)
