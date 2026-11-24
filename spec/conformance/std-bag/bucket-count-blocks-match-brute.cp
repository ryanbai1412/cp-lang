// expect: ok, output "YES"
// Range lengths around the power-of-two block sizes: 1, a square, one past it, and uneven ones.
use std.bag.{BucketCount}
for len in [1, 2, 3, 15, 16, 17, 63, 64, 65, 100, 257]:
    let lo = rand(-50, 50)
    var c = BucketCount(lo..lo + len)
    var plain: List<Int> = List.filled(len, 0)
    var n = 0
    for _ in 0..300:
        let x = lo + rand(0, len - 1)
        if rand(0, 2) == 0 and plain[x - lo] > 0:
            c.remove(x)
            plain[x - lo] -= 1
            n -= 1
        else:
            c.add(x)
            plain[x - lo] += 1
            n += 1
        let q = lo + rand(-2, len + 2)
        var lt = 0
        for v in 0..len:
            if lo + v < q: lt += plain[v]
        assert c.count_lt(q) == lt and c.len == n
        let k = rand(0, n - 1)
        var (acc, v) = (0, 0)
        while acc + plain[v] <= k:
            acc += plain[v]
            v += 1
        assert c.kth(k) == lo + v
    var all: List<Int> = []
    for v in 0..len:
        for _ in 0..plain[v]: all.push(lo + v)
    assert c.each().collect() == all
print("YES")
