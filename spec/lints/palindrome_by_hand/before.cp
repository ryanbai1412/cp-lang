let s: Str = read()
let n = s.len
var ok = true
for i in 0..n / 2:
    if s[i] != s[n - 1 - i]:
        ok = false
        break
print(yes(ok))
