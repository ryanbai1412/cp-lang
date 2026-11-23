// expect: ok, output "5 9"
// rule: 08.4.4
// A row fn reads its named field in O(1) and leaves the records, extra fields and all, as they were.
fn total<R: (w: Int, ..)>(xs: List<R>) -> Int = xs.map(r => r.w).sum()
let a = [(w: 2, u: 9), (w: 3, u: 1)]
print(total(a), a[0].u)
