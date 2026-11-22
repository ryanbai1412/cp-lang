// EGOI '22 P1 (SubsetMex, CP Lang 0.2.2): fewest insertions so that repeatedly
// replacing a subset by its mex can produce n.
// original: corpus/submissions/5198974.cpp
@multitest
let n: 1..=50 = read()
let have: [0..n]Int = read()
var need: [0..=n]Int = 0
need[n] = 1
var ans = 0
for i in (0..=n).rev:
    let short = need[i] - (if i < n: have[i] else: 0)
    if short > 0:
        ans += short
        for j in 0..i: need[j] += short
print(ans)
