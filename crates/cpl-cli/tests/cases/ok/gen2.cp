// cp: 0.1 329fa7d+dirty
let x: 1..=1000 = read()
fn divisors(n: 1..=1000) -> yields Int:
    var d = 1
    while d <= n:
        if n % d == 0:
            yield d
        d += 1
print(any(d > 1 and d * d == x for d in divisors(x)), all(d <= 10 for d in divisors(x)), count(true for d in divisors(x)), max(d for d in divisors(x) if d < x))
