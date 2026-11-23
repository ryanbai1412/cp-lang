let s: Str = read()
let n = s.len
var pi: List<Int> = [0 for _ in 0..n]
for i in 1..n:
    var j = pi[i - 1]
    while j > 0 and s[i] != s[j]:  // expect: hand_rolled_string_algo
        j = pi[j - 1]
    if s[i] == s[j]:
        j += 1
    pi[i] = j
print(pi)
