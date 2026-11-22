// expect: ok, output "5 1 5"
// rule: 08.4.1
fn total<R: (w: Int, ..)>(xs: List<R>) -> Int = xs.map(r => r.w).sum()
let a = (u: 1, w: 2)
let b = (..a, w: 5)
print(total([(w: 2, u: 1), (w: 3, u: 0)]), b.u, b.w)
