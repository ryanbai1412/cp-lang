@!multitest
let n: Int = read()
var pref: [0..=(2 * n)]?(Int, Int) = none
for i in 0..n:
    let (a, b): (Int, Int) = read()
    let c = (b - 1, i)
    if let p = pref[a - 1]:
        if c > p: pref[a - 1] = c
    else:
        pref[a - 1] = c
var ans: List<Int> = []
for x in pref:
    if let p = x: ans.push(p.1 + 1)
print(ans.len)
print(ans)
