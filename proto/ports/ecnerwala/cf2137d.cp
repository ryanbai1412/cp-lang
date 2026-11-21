@!multitest
let n: Int = read()
let b: [0..n]Int = read()
var val: [0..=n]?(id: Int, rem: Int) = none
var nxt = 0
var a: [0..n]Int = 0
for (i, x) in b.items:
    let (id, rem) = if let cur = val[x]:
        cur
    else:
        nxt += 1
        (nxt, x)
    a[i] = id
    val[x] = if rem == 1: none else: (id: id, rem: rem - 1)

if all(v == none for v in val): print(a) else: print(-1)
