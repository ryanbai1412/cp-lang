let (s, t): (Str, Str) = read()
var d: [0..=s.len, 0..=t.len]Int = [0..=s.len, 0..=t.len] (i, j) => i + j
for i in 1..=s.len:
    for j in 1..=t.len:
        d[i, j] = min(
            d[i - 1, j] + 1,
            d[i, j - 1] + 1,
            d[i - 1, j - 1] + (if s[i - 1] == t[j - 1]: 0 else: 1),
        )
print(d[s.len, t.len])
