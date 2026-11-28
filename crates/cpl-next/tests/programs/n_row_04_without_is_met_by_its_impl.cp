// expect: ok, output "7"
// rule: R08.4.1
// A `without` form's trait is met by the impl made for each record shape, not read off fields
// as a row bound is, so a written record meets it.
fn strip<R: (u: Int, ..)>(r: R) -> (..R without (u)) = (..r without (u))
print(strip((u: 1, w: 7)).w)
