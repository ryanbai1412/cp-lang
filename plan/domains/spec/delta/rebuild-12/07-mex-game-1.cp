// 1944C MEX Game 1
// in: t; per test n, a_1..a_n (0 <= a_i < n). out: final MEX under optimal play, per test
@!multitest
let n: Int = read()
let a: [n]Int = read()
var cnt: [n + 1]Int = 0
for v in a:
    cnt[v] += 1
var ones = 0
for v in 0..=n:
    if cnt[v] == 1:
        ones += 1
    if cnt[v] == 0 or ones == 2:
        print(v)
        break
