let s: Str = read()
var out = ""
var i = 0
while i < s.len:
    var j = i
    while j < s.len and s[j] == s[i]: j += 1
    out.push(s[i])
    out += "{j - i}"
    i = j
print(out)
