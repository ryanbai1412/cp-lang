// expect: L0516
let s = "ababa"
var pi: [0..5]Int = 0
for i in 1..s.len: // <- here
    var j = pi[i - 1]
    while j > 0 and s[i] != s[j]: j = pi[j - 1]
    if s[i] == s[j]: j += 1
    pi[i] = j
print(pi)
