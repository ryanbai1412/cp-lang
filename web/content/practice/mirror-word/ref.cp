let s: Str = read()
let n = s.len
print(yes((0..n).all(k => s[k] == s[n - 1 - k])))
