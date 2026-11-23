let s: Str = read()
let t: Str = read()
let n = s.len
var same = 0
for i in 0..n / 2:
    if s[i] == t[n - 1 - i]:
        same += 1
print(same)
