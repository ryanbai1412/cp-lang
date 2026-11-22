// expect: ok, output "1 2 4 4\n1 none 0 none none 2 none 1 none 1\n0\nYES"
use std.seq.{lis, cartesian_tree}
domain T5 = 0..5
domain T3 = 0..3
let a: [T5]Int = [3, 1, 2, 2, 5]
print(a.lis(), a.lis(strict: false).len)
let b: [T3]Int = [3, 1, 2]
let t = b.cartesian_tree()
print(t.root, t.left, t.right, t.parent)
print(b.cartesian_tree(min_root: false).root)
for _ in 0..200:
    domain N = 0..8
    let c: [N]Int = [N] _ => rand(0, 4)
    let s = c.lis()
    var best = 0
    for mask in 0..256:
        var (ok, last, cnt) = (true, -1, 0)
        for i in N:
            if (mask >> i.ord) & 1 == 1:
                if c[i] <= last: ok = false
                last = c[i]
                cnt += 1
        if ok and cnt > best: best = cnt
    assert s.len == best
    for k in 1..s.len: assert s[k - 1] < s[k] and c[s[k - 1]] < c[s[k]]
print("YES")
