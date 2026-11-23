let s: Str = read()
let t: Str = read()
var best: [0..=s.len, 0..=t.len]Int = 0
for i in 1..=s.len:
    for j in 1..=t.len:
        best[i, j] = if s[i - 1] == t[j - 1]: best[i - 1, j - 1] + 1 else: max(best[i - 1, j], best[i, j - 1])
print(best[s.len, t.len])
