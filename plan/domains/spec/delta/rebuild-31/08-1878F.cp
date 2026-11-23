// CF 1878F Vasilije Loves Number Theory
// d(n a) = d(n) d(a) for coprime a, and d(a) can be any positive integer
// (a prime power p^(e-1) with p not dividing n), so the answer is
// "d(n) divides n". n is kept as its factorization.
let sv = Sieve(1e6)

fn factored(x: Int) -> Counter<Int, Int>:
    var c = Counter<Int>(0)
    for (p, e) in sv.factor(x): c[p] += e
    c

@!multitest
let (n, q): (1..=1e6, 1..=1000) = read()
var cur = factored(n)
for _ in 0..q:
    if read<1..=2>() == 2:
        cur = factored(n)
        continue
    for (p, e) in sv.factor(read<1..=1e6>()): cur[p] += e
    var d = 1
    for (_, e) in cur.items: d *= e + 1
    var r = 1 % d
    for (p, e) in cur.items:
        for _ in 0..e: r = r * p % d
    print(yes(r == 0))
