fn ask(s: Str) -> List<Int>:
    print("?", s)
    flush()
    let k: Int = read()
    [read::<Int>() - 1 for _ in 0..k]

@!multitest
let n: Int = read()
let a: List<Int> = [read::<Int>() - 1 for _ in 0..n]
let vr = ask("R")
let vrl = ask("RL")
let vl = ask("L")

var ans: [0..n]?Int = none
var idx: [0..(4 * n)]?Int = none
for i in 0..n: idx[a[i]] = i
for z in 0..vrl.len:
    let i = idx[vrl[z]]!
    ans[i] = vr[z] - vrl[z]

var vl_has: [0..(4 * n)]Bool = false
for v in vl:
    let p = v + 1
    if 0 <= p and p < 4 * n: vl_has[p] = true

var bad = false
for i in (0..n).rev:
    if ans[i] == none:
        if ans[i + 1] == 0 and a[i + 1] == a[i] + 1:
            if i > 0 and ans[i - 1] == 0 and a[i - 1] == a[i] - 1:
                bad = true
                break
            ans[i] = if vl_has[a[i]]: 1 else: 2
        else:
            ans[i] = 2

if bad:
    print("!", -1)
else:
    print("!", [ans[i]! for i in 0..n])
flush()
