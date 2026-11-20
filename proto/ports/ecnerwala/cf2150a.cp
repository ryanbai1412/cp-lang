@!multitest
let (n, m): (Int, Int) = read()
let s: Str = read()
let a: [0..m]Int = read()
var black: Map<Int, Bool> = Map()
for x in a: black[x] = true
var cur = 1
for ch in s:
    if ch == 'A':
        cur += 1
        black[cur] = true
    else:
        cur += 1
        while cur in black: cur += 1
        black[cur] = true
        while cur in black: cur += 1
print(black.len)
print([k for (k, _) in black])
