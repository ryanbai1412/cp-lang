let s = "ababa"
var run = 0
for i in 1..s.len:
    if s[i] != s[i - 1]: run += 1
print(run)
