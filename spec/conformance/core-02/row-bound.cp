// expect: ok, output "3"
// rule: 02.5.4
fn span<R: (lo: Int, hi: Int, ..)>(r: R) -> Int = r.hi - r.lo
print(span((lo: 1, hi: 4, w: 9)))
