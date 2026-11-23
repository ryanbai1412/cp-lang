// expect: ok, output "YES 0 9 16\n0 -5 -8 -9 -7 -3"
fn best(lam: Int) -> (val: Int, count: Int):
    var b = (val: 3 * 3, count: 0)
    for c in 1..=10:
        let v = (c - 3) * (c - 3) + lam * c
        if v < b.val: b = (val: v, count: c)
    b
print(yes(aliens(5, best) == 4), aliens(3, best), aliens(0, best), aliens(7, best, lambdas: -100..=100))
fn pick(lam: Int) -> (val: Int, count: Int):
    var (v, c) = (0, 0)
    for x in [-5, -3, -1, 2, 4]:
        if x + lam < 0:
            v += x + lam
            c += 1
    (val: v, count: c)
print([aliens(k, pick, lambdas: -21..=21) for k in 0..=5])
