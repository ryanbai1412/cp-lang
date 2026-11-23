// 1950F 0, 1, 2, Tree!: fill levels breadth-first, two-child vertices first.
@!multitest
let a: 0..=1e5 = read()
let b: 0..=1e5 = read()
let c: 0..=1e5 = read()
if a + 1 != c:
    print(-1)
    return
var cur = 1
var nxt = 0
var h = 0
for i in 0..a + b:
    if cur == 0:
        h += 1
        cur = nxt
        nxt = 0
    cur -= 1
    nxt += if i < a: 2 else: 1
print(if a + b > 0: h + 1 else: 0)
