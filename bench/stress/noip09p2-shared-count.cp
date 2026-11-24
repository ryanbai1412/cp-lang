// NOIP 2009 Hankson (proto/ports/noip09p2.cp), with a planted multitest
// leak: the count lives above the `@!multitest` divider, so it is never
// reset between tests.
// class: multitest-leak

var found = 0  // planted
fn hankson(a0: Int, a1: Int, b0: Int, b1: Int) -> Int:
    for x in divisors(b1):
        if gcd(x, a0) == a1 and lcm(x, b0) == b1: found += 1
    found

@alt(hankson)
fn hankson_slow(a0: Int, a1: Int, b0: Int, b1: Int) -> Int =
    divisors(b1).count(x => gcd(x, a0) == a1 and lcm(x, b0) == b1)

@!multitest
let (a0, a1, b0, b1): (1..=2e9, 1..=2e9, 1..=2e9, 1..=2e9) = read()
print(hankson(a0, a1, b0, b1))
