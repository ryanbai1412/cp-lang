let (s, t): (Str, Str) = read()
var d: [0..=s.len, 0..=t.len]Int = 0
for i in 0..=s.len:
    for j in 0..=t.len:
        if i == 0 or j == 0: d[i, j] = i + j
        else:
            d[i, j] = min(d[i - 1, j] + 1, d[i, j - 1] + 1, d[i - 1, j - 1] + (if s[i - 1] == t[j - 1]: 0 else: 1))
print(d[s.len, t.len])
