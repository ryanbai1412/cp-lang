// expect: ok, output "YES YES"
// A `Semiring(max, +)` product of `?Int` matrices, `None` the zero, is the one the loop over `?Int` gives, `**` too.
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
let hi = Mat(a, over: Semiring(max, +)) * Mat(b, over: Semiring(max, +))
domain V = 0..3
let d: [V, V]?Int = [V, V] (i, j) => if i == j: None else: Some(i.ord - 2 * j.ord)
let sq = Mat(d, over: Semiring(max, +)) ** 2
let by_hand = Mat(d, over: Semiring(max, +)) * Mat(d, over: Semiring(max, +))
print(yes(hi.data == brute(true)), yes(sq.data == by_hand.data))
