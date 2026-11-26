// expect: ok, output "3 2\n1 2 1 2 9\nab 5\nYES"
// rule: R05.2.2, R05.5.1, R08.1.6
// `.clone()` is core's `Clone`: lists, strings and optionals write it, structs and records derive
// it, and a clone is owned, so a fn may return a clone of its parameter.
struct P:
    xs: List<Int>
    k: Int

fn grown(xs: List<Int>) -> List<Int>:
    var ys = xs.clone()
    ys.push(9)
    ys

let a = [1, 2]
let p = P(a.clone(), 5)
let q = p.clone()
print(grown(a).len, q.xs.len)
print(a, grown(a))
let r = (s: "ab", n: q.k).clone()
print(r.s, r.n)
print(yes(Some([4]).clone() == Some([4])))
