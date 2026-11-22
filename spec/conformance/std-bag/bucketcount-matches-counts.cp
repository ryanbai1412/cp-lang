// expect: ok, output "2 3 1 3 3 7 7 0\n1 NO\n0 0 YES\nYES"
use std.bag.{BucketCount}
var b = BucketCount(0..10)
b.add(3)
b.add(7, 2)
print(b.count(7), b.len, b.count_lt(7), b.count_lt(8), b.kth(0), b.kth(1), b.kth(2), b.count(99))
b.remove(7)
print(b.count(7), yes(b.is_empty))
b.clear()
print(b.count(3), b.len, yes(b.is_empty))
var c = BucketCount(5..42)
var plain: List<Int> = List.filled(37, 0)
var n = 0
for _ in 0..500:
    let x = rand(5, 41)
    if rand(0, 2) == 0 and plain[x - 5] > 0:
        c.remove(x)
        plain[x - 5] -= 1
        n -= 1
    else:
        c.add(x)
        plain[x - 5] += 1
        n += 1
    let q = rand(0, 50)
    var lt = 0
    for v in 5..min(max(q, 5), 42): lt += plain[v - 5]
    assert c.count_lt(q) == lt and c.len == n
    if n > 0:
        let k = rand(0, n - 1)
        var (acc, v) = (0, 5)
        while acc + plain[v - 5] <= k:
            acc += plain[v - 5]
            v += 1
        assert c.kth(k) == v
print("YES")
