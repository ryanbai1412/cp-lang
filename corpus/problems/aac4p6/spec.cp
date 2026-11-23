// aac4p6: written from the accepted solution (corpus/submissions/4227368.cpp).
let (n, k) = read<(1..=1_000_000, 0..=1_000_000_000)>()
var a: [0..n]Int = read()
let m = min(n, k)
if k > 0:
    var ds = Deque<Int>()
    var c = 0
    for j in 0..m:
        c += a[n - 1 - j]
        if c > 0:
            c -= 1
        else:
            ds.push(j)
    for i in 0..n:
        let p = a[i]
        ds.pop_while(b => b >= m - i)
        a[i] = max(a[i] - ds.len, 0)
        if ds.last != Some(m - i - 1):
            a[i] = max(a[i], 1)
        ds.push_front(-i - 1)
        for _ in 0..min(p, ds.len):
            ds.pop_front()
    if k > n and a.min()! == 0:
        a.rotate_right(k % n)
print(a)
