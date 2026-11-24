// The product of a list modulo 998244353, with a planted wrong modulus: the
// fast fn reduces by 1e9 + 7, the other contest prime.
// class: modulus
const MOD = 998244353

fn product(a: List<Int>) -> Int:
    var r = 1
    for x in a: r = r * x % (1e9 + 7)  // planted
    r

@alt(product)
fn product_slow(a: List<Int>) -> Int:
    var r = 1
    for x in a: r = r * x % MOD
    r

let n: 1..=100 = read()
let a: [0..n]1..=1e9 = read()
print(product([x for x in a]))
