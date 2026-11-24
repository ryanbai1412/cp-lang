// expect: ok, output "YES"
// A `Semiring(min, +)` product of `?Int` matrices, `None` the zero, is the one the loop over `?Int` gives.
domain R = 0..4
domain K = 0..5
domain C = 0..3
let a: [R, K]?Int = [R, K] (i, k) => if (i.ord * 7 + k.ord * 3) % 4 == 0: None else: Some(
    (i.ord * 13 + k.ord * 5) % 11 - 6
)
let b: [K, C]?Int = [K, C] (k, j) => if (k.ord + 2 * j.ord) % 3 == 1: None else: Some(
    (k.ord * 17 + j.ord * 7) % 9 - 3
)
fn brute(most: Bool) -> List<?Int>:
    var out: List<?Int> = []
    for i in R:
        for j in C:
            var best: ?Int = None
            for k in K:
                if let x = a[i, k]:
                    if let y = b[k, j]:
                        let s = x + y
                        best = if let o = best: Some(
                            if most: max(o, s) else: min(o, s)
                        ) else: Some(s)
            out.push(best)
    out
let lo = Mat(a, over: Semiring(min, +)) * Mat(b, over: Semiring(min, +))
print(yes(lo.data == brute(false)))
