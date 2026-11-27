// expect: ok, output "6 4\n21"
// rule: R04.2.11 R10.2.3
// input: "1 2 3\n4 5 6\n"
// `[R, C]Int` reads `R.count * C.count` tokens row-major: a generic body walks and sizes its
// domain `D` through `D`'s own `Domain` impl, so `D` may be a product.
domain R = 0..2
domain C = 0..3
let g = read<[R, C]Int>()
print(g[R(1), C(2)], (R, C).count(p => g[p] > 2))
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for v in D:
        s += a[v]
    return s
print(total(g))
