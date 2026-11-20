@!multitest
let n: Int = read()
let s: Str = read()
var good = true
var i = 0
while i < n:
    var j = i + 1
    while j < n and s[j] != s[j - 1]: j += 1
    if s[i] == '1' and (j - i) % 4 == 3:
        good = false
    i = j
print(if good: "YES" else: "NO")
