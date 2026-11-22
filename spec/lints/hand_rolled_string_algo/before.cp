let s = "ababa"
var pi = List.filled(s.len, 0)
for i in 1..s.len:
    var j = pi[i - 1]
    while j > 0 and s[i] != s[j]: j = pi[j - 1]
    if s[i] == s[j]: j += 1
    pi[i] = j
print(pi)
