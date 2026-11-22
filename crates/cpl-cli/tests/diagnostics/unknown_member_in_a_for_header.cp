let s: Str = read()
var n = 0
for (i, c) in s.rev.item:
    if c == 0:
        n += i
print(n)
