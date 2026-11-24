// expect: E0704
// rule: 08.4.1
fn total<R: (w: Int, ..)>(xs: List<R>) -> Int = xs.map(r => r.w).sum()
print(total([(u: 1)])) // <- here
