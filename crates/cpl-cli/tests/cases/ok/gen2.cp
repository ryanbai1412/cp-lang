// cp: 0.1 329fa7d+dirty
let x: 1..=1000 = read()
fn divisors(n: 1..=1000) -> yields Int:
    var d = 1
    while d <= n:
        if n % d == 0:
            yield d
        d += 1
print(yes(divisors(x).any(d => d > 1 and d * d == x)), yes(divisors(x).all(d => d <= 10)), count(divisors(x)), [d for d in divisors(x) if d < x].max()!)
