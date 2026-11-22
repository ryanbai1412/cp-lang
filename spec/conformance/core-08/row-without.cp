// expect: ok, output "7"
// rule: 08.4.1
fn strip<R: (u: Int, ..)>(r: R) -> (..R without (u)) = (..r without (u))
let s = strip((u: 1, w: 7))
print(s.w)
