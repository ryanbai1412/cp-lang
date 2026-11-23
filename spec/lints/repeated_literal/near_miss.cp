let a = [3, 4]
print(a.sum() % 998244353)
print(a[0] * a[1])
let (n, k) = read<(1..=200_000, 0..=1_000_000)>()
let h = read<[n]0..=1_000_000>()
enum Op:
    Set(d: -1_000_000_000..=1_000_000_000)
    Ask(h: 0..=1_000_000_000)

@test
fn the_modulus_is_an_odd_prime():
    assert 998244353 % 2 == 1
    assert 998244353 > 1_000
