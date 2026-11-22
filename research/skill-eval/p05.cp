let (n, x) = read<(1..=2e5, 1..=1e9)>()
domain I = 0..n
let a = read<[I]1..=1e9>()
var l: ?I = I.first
var s = 0
var ans = 0
for r in I:
    s += a[r]
    while s > x:
        if let p = l:
            s -= a[p]
            l = p.next
    if s == x:
        ans += 1
print(ans)
