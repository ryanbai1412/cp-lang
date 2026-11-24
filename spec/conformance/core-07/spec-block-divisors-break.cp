// expect: ok, output "1"
// rule: 07.5.12
let n = 12
let lim = 5
fn divisors(n: Int) -> yields Int:
    for d in 1..=n.isqrt:
        if n % d == 0:
            yield d
            if d * d != n: yield n / d

for d in divisors(n):
    if d > lim: break           // break, continue, return, break 'outer, loop else
    print(d)
