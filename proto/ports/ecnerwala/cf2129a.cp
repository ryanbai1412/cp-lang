@!multitest
let n: Int = read()
var pref: [0..=2 * n]?(Int, Int) = none
for i in 0..n:
    let (a, b): (Int, Int) = read()
    pref[a - 1] max= (b - 1, i)

let ans = [p.1 + 1 for x in pref if let p = x]
print(ans.len)
print(ans)
