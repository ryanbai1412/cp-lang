@!multitest
let (n, m): (Int, Int) = read()
let s: Str = read()
let a: [0..m]Int = read()
var black = Set(a)

var cur = 1
for ch in s:
    cur += 1
    if ch == 'B':
        while cur in black: cur += 1
    black.insert(cur)
    if ch == 'B':
        while cur in black: cur += 1

print(black.len)
print(black)
