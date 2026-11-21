@!multitest
let n: Int = read()
let x: [0..n]Int = read()
var stk: [0..=n]?Int = none
stk[x[0]] = 0
var par: [0..n]?Int = none
var is_ext: [0..n]Bool = false
for i in 1..n:
    if x[i] > x[i - 1] + 1 or x[i] == 1:
        print("NO")
        return
    let p = stk[x[i] - 1]!
    par[i] = p
    if x[i] <= x[i - 1]: is_ext[p] = true
    stk[x[i]] = i

if any(is_ext[i] and is_ext[par[i]!] for i in 1..n):
    print("NO")
    return

var nxt: [0..n]?Int = none
var lst: [0..n]?(Int, Int) = none

fn merge(a: ?(Int, Int), b: ?(Int, Int)) -> ?(Int, Int):
    let (ah, at) = a ?? return b
    let (bh, bt) = b ?? return a
    nxt[at] = bh
    (ah, bt)

for i in (0..n).rev:
    lst[i] = if is_ext[i]: merge((i, i), lst[i]) else: merge(lst[i], (i, i))
    if let p = par[i]: lst[p] = merge(lst[i], lst[p])

var res: [0..n]Int = 0
var cur: ?Int = lst[0]!.0
var v = 1
while let c = cur:
    res[c] = v
    v += 1
    cur = nxt[c]
print("YES")
print(res)
