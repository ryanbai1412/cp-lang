// expect: ok, output "1 2"
// rule: 08.1.9
// `impl<domain D> Tr for D` beside `impl Tr for F64`: a call picks the impl from the type it is given.
trait Searchable:
    type Found
    fn tag() -> Int
impl Searchable for F64:
    type Found = ?F64
    fn tag() -> Int = 1
impl<domain D> Searchable for D:
    type Found = Int
    fn tag() -> Int = 2
fn look<D: Searchable>(x: D) -> Int = D.tag()
domain V = 0..4
print(look(0.5), look(V(3)))
