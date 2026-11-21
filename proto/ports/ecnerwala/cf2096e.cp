@! multitest
let n: Int = read()
let s: Str = read()
var cnt_b = 0
var cnt_p_odd = 0
var ans = 0
for i in 0..n:
    let c = s[n - 1 - i]
    if c == 'P':
        ans += cnt_b
        cnt_p_odd += i & 1
    else:
        cnt_b += 1
ans += abs(cnt_p_odd - (n - cnt_b) / 2)
print(ans / 2)
