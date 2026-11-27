// expect: E0203
// `..R without (q)` drops a field `R`'s row bound does not have (R08.4.4).
fn strip<R: (u: Int, ..)>(r: R) -> (..R without (q)) = todo // <- here
print(strip((u: 1, w: 7)))
