let s = "abca"
var cnt: [0..26]Int = 0
for c in s:
    cnt[c - 'a'] += 1
print(cnt[0])
