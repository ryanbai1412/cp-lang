@!multitest
let s: Str = read()
var cnt = 0
for c in s:
    if c == 'Y': cnt += 1
print(if cnt <= 1: "YES" else: "NO")
