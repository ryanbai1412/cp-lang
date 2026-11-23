domain Letter = 'a'..='z'
let s = "abca"
var cnt: [Letter]Int = 0
for c in s:
    cnt[Letter(c)] += 1
print(cnt[Letter('a')])
