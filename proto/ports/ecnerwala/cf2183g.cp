@!interactive

fn ask(s: Str) -> List<Int>:
    print("?", s)
    let k: Int = read()
    let got: [0..k]Int = read()
    [x - 1 for x in got]

@!multitest
let n: Int = read()
let a_in: [0..n]Int = read()
let a = [0..n] i => a_in[i] - 1
let vr = ask("R")
let vrl = ask("RL")
let vl = ask("L")

var ans: [0..n]?Int = none
var idx: [0..4 * n]?Int = none
for (i, x) in a.items: idx[x] = i
for (v, r) in zip(vrl, vr): ans[idx[v]!] = r - v

var vl_has: [0..4 * n]Bool = false
for v in vl if 0 <= v + 1 < 4 * n: vl_has[v + 1] = true

for i in (0..n).rev if ans[i] == none:
    if ans[i + 1] == 0 and a[i + 1] == a[i] + 1:
        if i > 0 and ans[i - 1] == 0 and a[i - 1] == a[i] - 1:
            print("!", -1)
            return
        ans[i] = if vl_has[a[i]]: 1 else: 2
    else:
        ans[i] = 2

print("!", [x! for x in ans])
