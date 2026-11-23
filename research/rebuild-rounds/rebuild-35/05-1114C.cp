// CF 1114C Trailing Loves: min over prime powers p^e || b of legendre(n, p) / e.
let n: 1..=1e18 = read()
let b: 2..=1e12 = read()
fn legendre(n: Int, p: Int) -> Int:
    var count = 0
    var q = n
    while q > 0:
        q /= p
        count += q
    count
var rest = b
var best: ?Int = None
var p = 2
while p * p <= rest:
    if rest % p == 0:
        var e = 0
        while rest % p == 0:
            rest /= p
            e += 1
        best min= legendre(n, p) / e
    p += 1
if rest > 1: best min= legendre(n, rest)
print(best!)
