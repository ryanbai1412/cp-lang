// expect: ok, output "1\n12\n2\n6\n3\n4"
// rule: 05.7.3
fn divisors(n: Int) -> yields Int:
    for d in 1..=n.isqrt():
        if n % d == 0:
            yield d
            if d * d != n: yield n / d
let gs = List(divisors(12), divisors(30))
for d in gs[0]: print(d)
