let s: Str = read()
var total = 0
var run = 0
for c in s:
    if c == 'f':
        run += 1
    else:
        total += run / 2
        run = 0
total += run / 2
print(total)
