let (a, e): (0..=1000000000, 0..=1000000000000000000) = read()
fn power(b0: Int, e0: Int) -> Int:
    var b = b0 % 998244353
    var x = e0
    var r = 1
    while x > 0:
        if x % 2 == 1:
            r = r * b % 998244353
        b = b * b % 998244353
        x /= 2
    return r
print(power(a, e))
