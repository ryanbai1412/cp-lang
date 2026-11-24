// expect: ok, output "YES YES YES YES"
// A max-plus product's entries read, change and multiply again as any `Mat`'s: setting one, or
// adding to one through `at`, is seen by the next product.
domain V = 0..3
let d: [V, V]?Int = [V, V] (i, j) => if (i.ord + j.ord) % 2 == 0: Some(i.ord * 3 - j.ord) else: None
let m = Mat(d, over: Semiring(max, +))
fn brute(a: List<?Int>, b: List<?Int>, n: Int) -> List<?Int>:
    var out: List<?Int> = List.filled(n * n, None)
    for i in 0..n:
        for j in 0..n:
            for k in 0..n:
                if let x = a[i * n + k]:
                    if let y = b[k * n + j]: out[i * n + j] = Some(
                        max(out[i * n + j] ?? (x + y), x + y)
                    )
    out
var p = m * m
let first = p.data == brute(m.data, m.data, 3)
p.set(0, 1, Some(40))
let q = p * m
var r = m * m
if let v = r.get(1, 1): r.at(1, 1) = Some(v + 7)
let s = r * m
print(
    yes(first),
    yes(p.get(0, 1) == Some(40)),
    yes(q.data == brute(p.data, m.data, 3)),
    yes(s.data == brute(r.data, m.data, 3)),
)
