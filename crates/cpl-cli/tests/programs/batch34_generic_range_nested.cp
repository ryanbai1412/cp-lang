fn halves<D>(a: [D]Int) -> Int:
    var total = 0
    for i in D: total += a[i]
    if D.len > 1:
        let half = D.len / 2
        let left: [0..half]Int = [0..half] i => a[i] + 1
        total += halves(left)
    for i in D: total += a[i]
    total + D.len

fn any<D>(mut d: [D]Int, k: Int):
    d[k] += 1

var xs: [0..6]Int = [1, 2, 3, 4, 5, 6]
print(halves(xs))
any(mut xs, 5)
print(xs)
