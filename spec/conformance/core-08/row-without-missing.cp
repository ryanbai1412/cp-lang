// expect: E0203 at 1
// rule: 08.4.4
fn strip<R: (u: Int, ..)>(r: R) -> (..R without (q)) = todo
