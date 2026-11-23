// CF 1978E Computing Machine: apply every op-1 first, then every op-2; interior answers
// come from the whole-string pass, the two cells at each end of a query are recomputed.
// Strings are 0-based, query bounds 1-based.
@!multitest
let n = read<1..=2e5>()
let s = read<Str>()
let t = read<Str>()

// b after op 1 inside the window lo..=hi
fn b1(lo: Int, hi: Int, j: Int) -> Bool =
    t[j] == '1' or (lo < j and j < hi and s[j - 1] == '0' and s[j + 1] == '0')
// a after op 2 inside the window lo..=hi
fn a2(lo: Int, hi: Int, i: Int) -> Bool =
    s[i] == '1' or (lo < i and i < hi and b1(lo, hi, i - 1) and b1(lo, hi, i + 1))

var pre: [0..=n]Int = 0
for i in 0..n: pre[i + 1] = pre[i] + Int(a2(0, n - 1, i))
let q = read<1..=2e5>()
repeat q:
    let l = read<1..=n>() - 1
    let r = read<1..=n>() - 1
    var ans = 0
    if r - l >= 4: ans += pre[r - 1] - pre[l + 2]
    for i in l..=min(r, l + 1): ans += Int(a2(l, r, i))
    for i in max(l + 2, r - 1)..=r: ans += Int(a2(l, r, i))
    print(ans)
