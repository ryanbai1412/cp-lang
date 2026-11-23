// CF 1999D Slavic's Exam
// in: T; per test s (letters and '?'), t. out: NO, or YES then s with each '?' replaced.
@!multitest
var s: Str = read()
let t: Str = read()
var j = 0
for i in s.keys:
    if j < t.len and (s[i] == '?' or s[i] == t[j]):
        s[i] = t[j]
        j += 1
    elif s[i] == '?':
        s[i] = 'a'
print(yes(j == t.len))
if j == t.len: print(s)
